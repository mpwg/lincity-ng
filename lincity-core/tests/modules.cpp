#include "lincity/ConstructionReport.hpp"
#include "lincity/Vehicles.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/blacksmith.hpp"
#include "lincity/modules/market.hpp"
#include "lincity/modules/mill.hpp"
#include "lincity/modules/organic_farm.hpp"
#include "lincity/modules/residence.hpp"
#include "lincity/modules/solar_power.hpp"
#include "lincity/modules/power_line.hpp"
#include "lincity/modules/substation.hpp"
#include "lincity/modules/track_road_rail.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string_view>
#include <tuple>
#include <vector>

namespace {
class Report : public ConstructionReport {
public:
  int rows = 0;
  using Quantity = std::tuple<std::string, std::string, int, int>;
  std::vector<Quantity> quantities;
  std::vector<std::string> labels;
  std::vector<std::pair<std::string, double>> percentages;

  void addBlank() override { ++rows; }
  void addLabel(const std::string& label) override {
    labels.push_back(label);
    ++rows;
  }
  void addLabeledText(const std::string&, const std::string&) override { ++rows; }
  void addInteger(const std::string&, int) override { ++rows; }
  void addDecimal(const std::string&, double) override { ++rows; }
  void addPercentage(const std::string& label, double percentage) override {
    percentages.emplace_back(label, percentage);
    ++rows;
  }
  void addQuantity(const std::string&, int, int) override { ++rows; }
  void addPrefixedQuantity(const std::string& prefix, const std::string& label,
                           int value, int capacity) override {
    quantities.emplace_back(prefix, label, value, capacity);
    ++rows;
  }
};

void require(bool condition, const char* description) {
  if(!condition) throw std::runtime_error(description);
}

template<class Exception, class Operation>
void requireRejection(Operation operation, const char* description) {
  try {
    operation();
  } catch(const Exception&) {
    return;
  }
  throw std::runtime_error(description);
}

void verifyMissingAnimationResource() {
  World world(32);
  const MapPoint site(12, 12);
  roadConstructionGroup.placeItem(world, site);
  Vehicle car(world, site, VEHICLE_BLUECAR);
  world.bulldozeArea(site);
  const auto* tile = world.map(site);
  const auto* frames = tile->getFrames();
  require(tile->is_bare() && frames && frames->size() == 1,
          "road removal should leave its vehicle overlay on the tile");

  auto group = blacksmithConstructionGroup;
  group.resourceID = "UnknownSmithyAnimation";
  std::unique_ptr<Construction> rejected(group.createConstruction(world));
  requireRejection<std::out_of_range>([&] { rejected->place(site); },
                                      "unknown animation resource accepted");
  require(tile->is_bare() && !tile->getConstruction(),
          "missing animation resource left a construction in the map");
  require(tile->getFrames() == frames && frames->size() == 1 && &frames->front() == &*car.frameIt,
          "missing animation resource changed the existing vehicle overlays");
}

void verifyPlacementPreflight() {
  World world(32);
  const MapPoint site(12, 12);
  roadConstructionGroup.placeItem(world, site.se());
  const auto* road = world.map(site.se())->getConstruction();
  const auto initialCount = world.map.constructionCount();
  requireRejection<std::logic_error>([&] { blacksmithConstructionGroup.placeItem(world, site); },
                                    "occupied southeast footprint tile accepted");
  require(world.map(site)->is_bare() && world.map(site.se())->getConstruction() == road,
          "overlapping construction changed an occupied footprint");

  const MapPoint lakeshore(20, 20);
  world.map(lakeshore.se())->setTerrain(GROUP_WATER);
  requireRejection<std::logic_error>([&] { blacksmithConstructionGroup.placeItem(world, lakeshore); },
                                    "water inside an ordinary footprint accepted");
  require(world.map(lakeshore)->is_bare() && world.map(lakeshore.se())->is_lake(),
          "failed placement changed shore terrain");

  std::unique_ptr<Construction> tooCloseToEdge(blacksmithConstructionGroup.createConstruction(world));
  requireRejection<std::runtime_error>([&] { tooCloseToEdge->place(MapPoint(30, 30)); },
                                       "footprint crossed the map boundary");
  require(world.map(MapPoint(30, 30))->is_bare() && world.map(MapPoint(31, 31))->is_bare(),
          "failed boundary placement occupied tiles");
  require(world.map.constructionCount() == initialCount, "failed placement registered a building");
}

void verifyPlacementFactoryFailures() {
  struct UnavailableBlacksmith : BlacksmithConstructionGroup {
    UnavailableBlacksmith() : BlacksmithConstructionGroup(blacksmithConstructionGroup) {}

    Construction* createConstruction(World&) override {
      ++requests;
      return nullptr;
    }

    int requests = 0;
  } group;
  World world(32);
  const auto verifyEmpty = [&](MapPoint origin) {
    for(const auto p : {origin, origin.e(), origin.s(), origin.se()})
      require(world.map(p)->is_bare() && !world.map(p)->getFrames(),
              "failed placement left a construction or frame in its footprint");
    require(world.map.constructionCount() == 0 && world.stats.groupCount[GROUP_BLACKSMITH] == 0,
            "failed placement registered a construction");
  };

  const MapPoint edge(30, 30);
  requireRejection<std::runtime_error>([&] { group.placeItem(world, edge); },
                                       "direct placement on map boundary accepted");
  require(group.requests == 0, "boundary rejected only after calling the construction factory");
  verifyEmpty(edge);

  const MapPoint site(12, 12);
  requireRejection<std::runtime_error>([&] { group.placeItem(world, site); },
                                       "placement accepted a null construction factory result");
  require(group.requests == 1, "valid site did not reach the construction factory");
  verifyEmpty(site);
}

void verifyEvacuationFit() {
  for(bool farmTrades : {true, false}) {
    World world(32);
    const MapPoint site(12, 12);
    organic_farmConstructionGroup.placeItem(world, site);
    trackConstructionGroup.placeItem(world, site.w());
    auto& farm = *world.map(site)->getConstruction();
    auto& track = *world.map(site.w())->getConstruction();
    farm.commodityCount[STUFF_FOOD] = 200;
    track.commodityCount[STUFF_FOOD] = MAX_FOOD_ON_TRACK - 75;
    world.evacuateArea(site);
    require(farm.tellstuff(STUFF_FOOD, 500) == 200 && farm.tellstuff(STUFF_FOOD, -2) == 18,
            "evacuation did not distinguish available stock from the vehicle fill ratio");
    (farmTrades ? farm : track).trade();
    if(track.commodityCount[STUFF_FOOD] != MAX_FOOD_ON_TRACK || farm.commodityCount[STUFF_FOOD] != 125)
      throw std::runtime_error(std::string(farmTrades ? "farm" : "track") + " evacuation trade left food " +
        std::to_string(farm.commodityCount[STUFF_FOOD]) + " at farm and " +
        std::to_string(track.commodityCount[STUFF_FOOD]) + " at track");
    (farmTrades ? track : farm).trade();
    require(track.commodityCount[STUFF_FOOD] == MAX_FOOD_ON_TRACK && farm.commodityCount[STUFF_FOOD] == 125,
            "a full track accepted additional food during evacuation");
    require(farm.commodityProd[STUFF_FOOD] == 0 && track.commodityProd[STUFF_FOOD] == 0,
            "evacuation changed food production counters");
  }
}

void verifyTradeWithNoReceivingCapacity() {
  World world(32);
  const MapPoint site(12, 12);
  const auto homeSite = site.w(3);
  organic_farmConstructionGroup.placeItem(world, site);
  residenceLLConstructionGroup.placeItem(world, homeSite);
  auto& farm = *world.map(site)->getConstruction();
  auto& home = *world.map(homeSite)->getConstruction();
  world.evacuateArea(site);
  world.evacuateArea(homeSite);
  farm.commodityCount[STUFF_FOOD] = 200;
  home.commodityCount[STUFF_FOOD] = 75;
  farm.trade();
  home.trade();
  require(farm.commodityCount[STUFF_FOOD] == 200 && home.commodityCount[STUFF_FOOD] == 75,
          "two evacuating buildings traded into a network without receiving capacity");
}

void verifyTradeTaxesAndOutgoingCommuter() {
  World world(32);
  const MapPoint site(12, 12);
  const auto smithySite = site.w(2);
  const auto homeSite = site.n(3);
  roadConstructionGroup.placeItem(world, site);
  blacksmithConstructionGroup.placeItem(world, smithySite);
  residenceLLConstructionGroup.placeItem(world, homeSite);
  auto& road = *world.map(site)->getConstruction();
  auto& smithy = *world.map(smithySite)->getConstruction();
  auto& home = *world.map(homeSite)->getConstruction();
  world.vehicleTraffic = World::VehicleTraffic::Normal;
  struct Delivery { Commodity commodity; Construction& recipient; int supply, moved; };
  const Delivery deliveries[] = {
    {STUFF_LABOR, smithy, 5120, 615}, {STUFF_COAL, smithy, 512, 19},
    {STUFF_GOODS, home, 10240, 476}, {STUFF_FOOD, home, 8192, 891},
  };
  for(const auto& delivery : deliveries)
    road.commodityCount[delivery.commodity] = delivery.supply;
  {
    std::istringstream draws("LINCITY-SIMULATION-TRACE 1\nC 0\nZ\n");
    SimulationTrace trace(draws);
    road.trade();
    trace.finish();
  }
  for(const auto& [commodity, recipient, supply, moved] : deliveries) {
    require(recipient.commodityCount[commodity] == moved && road.commodityCount[commodity] == supply - moved,
            "road delivery changed stock by the wrong amount");
    require(recipient.commodityProd[commodity] == 0 && road.commodityProd[commodity] == 0,
            "trade was recorded as local production");
  }
  require(world.taxable.labor == 615 && world.taxable.coal == 19 && world.taxable.goods == 476,
          "consumer deliveries were recorded under the wrong tax bases");
  require(smithy.tellstuff(STUFF_COAL, 949) == -1 && smithy.tellstuff(STUFF_COAL, 950) == 950,
          "consumer offered coal to a less full source");

  // Open a lower-labor route after the delivery. An outgoing commuter can
  // move onto it while an incoming commuter would have no permissible route.
  trackConstructionGroup.placeItem(world, site.e());
  world.do_animate(1);
  const bool active = std::any_of(world.map.begin(), world.map.end(), [](const auto& tile) {
    const auto* frames = tile.getFrames();
    return frames && std::any_of(frames->begin(), frames->end(), [](const auto& frame) {
      return frame.resourceGroup && frame.resourceGroup->is_vehicle;
    });
  });
  require(active, "outgoing commuter did not follow the lower-labor route");
}

void verifyMarketInventoryIsUntaxed() {
  World world(32);
  const MapPoint site(12, 12);
  roadConstructionGroup.placeItem(world, site);
  marketConstructionGroup.placeItem(world, site.e());
  auto& road = *world.map(site)->getConstruction();
  auto& market = *world.map(site.e())->getConstruction();
  road.commodityCount[STUFF_GOODS] = 10240;
  road.commodityCount[STUFF_COAL] = 512;
  road.trade();
  require(market.commodityCount[STUFF_GOODS] == 1280 && road.commodityCount[STUFF_GOODS] == 8960,
          "market did not receive goods at the transport rate limit");
  require(market.commodityCount[STUFF_COAL] == 64 && road.commodityCount[STUFF_COAL] == 448,
          "market did not receive coal at the transport rate limit");
  require(world.taxable.goods == 0 && world.taxable.coal == 0 && world.taxable.labor == 0,
          "warehouse delivery was taxed as consumption");
}

void verifyTransportExchangeNearCapacity() {
  struct Case { int initialTrack, finalTrack, finalRail; };
  // Older builds could save over-capacity inventory after evacuation trade.
  for(const auto& test : {Case{2172, 1916, 32768}, Case{2000, 1792, 32720}}) {
    World world(32);
    const MapPoint site(12, 12);
    trackConstructionGroup.placeItem(world, site);
    railConstructionGroup.placeItem(world, site.w());
    trackConstructionGroup.placeItem(world, site.e());
    auto& center = *world.map(site)->getConstruction();
    auto& trackFood = center.commodityCount[STUFF_FOOD];
    auto& railFood = world.map(site.w())->getConstruction()->commodityCount[STUFF_FOOD];
    auto& eastFood = world.map(site.e())->getConstruction()->commodityCount[STUFF_FOOD];
    trackFood = test.initialTrack;
    railFood = MAX_FOOD_ON_RAIL;
    center.trade();
    require(railFood == test.finalRail, "full rails delivered beyond the receiving track's capacity");
    require(trackFood == test.finalTrack && eastFood == 256, "track did not drain into the available track");
    require(world.taxable.labor == 0 && world.taxable.coal == 0 && world.taxable.goods == 0,
            "stock redistribution across transport affected taxes");
  }
}

void verifyOldOverflowCannotSupplyEvacuatingNeighbor() {
  World world(32);
  const MapPoint site(12, 12);
  organic_farmConstructionGroup.placeItem(world, site);
  trackConstructionGroup.placeItem(world, site.w());
  auto& farm = *world.map(site)->getConstruction();
  auto& track = *world.map(site.w())->getConstruction();
  farm.commodityCount[STUFF_FOOD] = 200;
  track.commodityCount[STUFF_FOOD] = 2172;
  world.evacuateArea(site);
  track.trade();
  require(farm.commodityCount[STUFF_FOOD] == 200 && track.commodityCount[STUFF_FOOD] == 2172,
          "overfull track sent food back into an evacuating farm");
}

void verifyBlacksmith() {
  World world(32);
  const MapPoint site(12, 12);
  blacksmithConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->getConstruction();
  building.levelStuff(STUFF_COAL, BLACKSMITH_COAL_USED);
  building.levelStuff(STUFF_STEEL, BLACKSMITH_STEEL_USED);
  building.levelStuff(STUFF_LABOR, BLACKSMITH_LABOR - 1);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == 0, "blacksmith produced without labor");
  require(building.commodityCount[STUFF_STEEL] == BLACKSMITH_STEEL_USED,
          "idle blacksmith consumed steel");
  building.levelStuff(STUFF_LABOR, BLACKSMITH_LABOR);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == GOODS_MADE_BY_BLACKSMITH,
          "blacksmith produced the wrong amount");
  for(auto input : {STUFF_STEEL, STUFF_COAL, STUFF_LABOR})
    require(building.commodityCount[input] == 0, "blacksmith did not consume an input");
}

void verifyMill(Commodity fuel, int amount) {
  World world(32);
  const MapPoint site(12, 12);
  millConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->getConstruction();
  building.levelStuff(STUFF_FOOD, FOOD_USED_BY_MILL);
  building.levelStuff(STUFF_LABOR, MILL_LABOR);
  building.levelStuff(fuel, amount);
  building.update();
  require(building.commodityCount[STUFF_GOODS] == GOODS_MADE_BY_MILL,
          "mill produced the wrong amount");
  require(building.commodityCount[fuel] == 0, "mill did not consume its fuel");
}

void verifySolarPower(int tech, int output) {
  World world(32);
  world.tech_level = tech;
  const MapPoint site(12, 12);
  solarPowerConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->getConstruction();
  building.levelStuff(STUFF_LABOR, SOLAR_POWER_LABOR);
  building.update();
  require(building.commodityCount[STUFF_HIVOLT] == output, "solar output changed");
  building.levelStuff(STUFF_LABOR, SOLAR_POWER_LABOR);
  building.levelStuff(STUFF_HIVOLT, MAX_HIVOLT_AT_SOLARPS);
  building.update();
  require(building.commodityCount[STUFF_HIVOLT] == MAX_HIVOLT_AT_SOLARPS,
          "solar plant exceeded its capacity");
  require(building.commodityCount[STUFF_LABOR] == SOLAR_POWER_LABOR,
          "idle solar plant consumed labor");
}

void verifyInventoryAndProductionReport() {
  World world(32);
  const MapPoint site(12, 12);
  blacksmithConstructionGroup.placeItem(world, site);
  auto& building = *world.map(site)->getConstruction();
  building.commodityCount[STUFF_COAL] = BLACKSMITH_COAL_USED;
  building.commodityCount[STUFF_STEEL] = BLACKSMITH_STEEL_USED;
  building.commodityCount[STUFF_LABOR] = BLACKSMITH_LABOR;
  world.total_time = 99;
  building.update();

  Report production;
  building.report(production, true);
  const std::vector<Report::Quantity> expectedProduction{
    {"<-- ", "Goods", GOODS_MADE_BY_BLACKSMITH, building.commodityMaxProd[STUFF_GOODS]},
    {"--> ", "Labor", -BLACKSMITH_LABOR, -building.commodityMaxCons[STUFF_LABOR]},
    {"--> ", "Coal", -BLACKSMITH_COAL_USED, -building.commodityMaxCons[STUFF_COAL]},
    {"--> ", "Steel", -BLACKSMITH_STEEL_USED, -building.commodityMaxCons[STUFF_STEEL]}
  };
  require(production.quantities == expectedProduction, "wrong monthly production rows");

  world.evacuateArea(site);
  Report inventory;
  building.report(inventory, false);
  const std::vector<Report::Quantity> expectedInventory{
    {"<<< ", "Goods", GOODS_MADE_BY_BLACKSMITH, MAX_GOODS_AT_BLACKSMITH},
    {"<<< ", "Labor", 0, MAX_LABOR_AT_BLACKSMITH},
    {"<<< ", "Coal", 0, MAX_COAL_AT_BLACKSMITH},
    {"<<< ", "Steel", 0, MAX_STEEL_AT_BLACKSMITH}
  };
  require(inventory.quantities == expectedInventory, "wrong evacuating inventory rows");
  require(Construction::getStuffName(STUFF_COAL) == "Coal",
          "invalid commodity display name in the finance report");
}

void verifyTransportReport() {
  World world(32);
  const MapPoint site(12, 12);
  trackConstructionGroup.placeItem(world, site);
  roadConstructionGroup.placeItem(world, site.e());
  auto& track = *world.map(site)->getConstruction();
  auto& road = *world.map(site.e())->getConstruction();
  track.commodityCount[STUFF_FOOD] = 2048;

  for(const auto& [delivered, percentage] : {std::pair{256, 10.34592}, {512, 19.82968}}) {
    track.trade();
    require(track.commodityCount[STUFF_FOOD] == 2048 - delivered && road.commodityCount[STUFF_FOOD] == delivered,
            "traffic report fixture did not deliver food at the expected rate");

    Report activity;
    track.report(activity, true);
    require(activity.labels == std::vector<std::string>{"Track", "Traffic"} && activity.quantities.empty(),
            "transport activity report did not identify traffic separately from inventory");
    const auto food = std::find_if(activity.percentages.begin(), activity.percentages.end(), [](const auto& row) {
      return row.first == "Food";
    });
    require(food != activity.percentages.end() && std::abs(food->second - percentage) < 0.000001,
            "transport traffic report did not smooth food delivery across ticks");
    require(std::none_of(activity.percentages.begin(), activity.percentages.end(), [](const auto& row) {
      return row.first == "Hi-Volt";
    }), "transport traffic reported unsupported high-voltage flow");

    Report inventory;
    track.report(inventory, false);
    const Report::Quantity foodStock{"<-> ", "Food", 2048 - delivered, 2048};
    require(std::find(inventory.quantities.begin(), inventory.quantities.end(), foodStock) != inventory.quantities.end() &&
              inventory.percentages.empty(), "transport inventory report did not show remaining stock");
  }
}

void verifyResidenceFootprints() {
  for(auto* group : {&residenceLLConstructionGroup, &residenceMLConstructionGroup,
                    &residenceHLConstructionGroup, &residenceLHConstructionGroup,
                    &residenceMHConstructionGroup, &residenceHHConstructionGroup}) {
    World world(32);
    const MapPoint site(12, 12);
    group->placeItem(world, site);
    const MapPoint corner = site.e(group->size - 1).s(group->size - 1);
    require(world.map(site)->is_residence() && world.map(corner)->is_residence(),
            "residential footprint is not recognized");
    require(!world.map(site.w())->is_residence(), "a bare tile reported a residence");
    world.bulldozeArea(corner);
    require(!world.map(site)->is_residence() && !world.map(corner)->is_residence(),
            "removed residence kept part of its footprint");
    world.do_time_step();
    require(world.map.constructionCount() == 0, "detached residence was not pruned");
  }
}

void verifyDetachedConstructionLifetime() {
  struct ObservedSmithy : Blacksmith {
    ObservedSmithy(World& world, int& destructions)
      : Blacksmith(world, &blacksmithConstructionGroup), destructions(destructions) {}
    ~ObservedSmithy() override { ++destructions; }

    int& destructions;
  };

  int destructions = 0;
  {
    World world(32);
    const MapPoint site(12, 12);
    (new ObservedSmithy(world, destructions))->place(site);
    world.bulldozeArea(site);
    require(destructions == 0, "detached building deleted before the simulation boundary");
    (new ObservedSmithy(world, destructions))->place(site);
  }
  require(destructions == 2, "ending the city lost a detached construction");
}

void verifyPowerCableLimits() {
  World world(32);
  const MapPoint site(12, 12);
  const int size = substationConstructionGroup.size;
  substationConstructionGroup.placeItem(world, site);
  struct Cable { MapPoint water; MapPoint pole; int mask; unsigned flag; };
  const Cable cables[] = {
    {site.n(), site.n(2), 8, FLAG_POWER_CABLES_0},
    {site.w(), site.w(2), 4, FLAG_POWER_CABLES_90},
    {site.e(size), site.e(size + 1), 2, FLAG_POWER_CABLES_90},
    {site.s(size), site.s(size + 1), 1, FLAG_POWER_CABLES_0},
  };
  for(const auto& link : cables) {
    world.map(link.water)->setTerrain(GROUP_WATER);
    powerlineConstructionGroup.placeItem(world, link.pole);
  }

  world.map(site.ne())->setTerrain(GROUP_WATER);
  powerlineConstructionGroup.placeItem(world, site.n(2).e());
  world.map.connect_transport(8, 8, 18, 18);
  auto& station = *world.map(site)->getConstruction();
  for(const auto& link : cables) {
    require(station.countPowercables(link.mask) == 1,
            "power line did not link to the intended side");
    require(world.map(link.water)->flags & link.flag,
            "power line did not span the intended water tile");
  }
  require(!(world.map(site.ne())->flags & FLAG_POWER_CABLES_0),
          "two suspended cables were attached on the same side");

  world.bulldozeArea(site.n(2));
  require(station.countPowercables(8) == 1, "remaining north cable did not take over");
  require(world.map(site.ne())->flags & FLAG_POWER_CABLES_0, "north cable did not relocate");
  require(!(world.map(site.n())->flags & FLAG_POWER_CABLES_0), "removed north cable survived");
}

void verifyPowerlineRelay() {
  World world(32);
  const MapPoint site(12, 12);
  const auto plantSite = site.w(4);
  solarPowerConstructionGroup.placeItem(world, plantSite);
  powerlineConstructionGroup.placeItem(world, site);
  powerlineConstructionGroup.placeItem(world, site.e());
  world.map.connect_transport(6, 10, 16, 16);
  auto& plant = *world.map(plantSite)->getConstruction();
  auto& feeder = dynamic_cast<Powerline&>(*world.map(site)->getConstruction());
  auto& relay = dynamic_cast<Powerline&>(*world.map(site.e())->getConstruction());
  {
    std::istringstream noDraws("LINCITY-SIMULATION-TRACE 1\nZ\n");
    SimulationTrace trace(noDraws);
    feeder.trade();
    trace.finish();
  }
  require(feeder.anim_counter == 0 && relay.anim_counter == 0,
          "a generator without stored power started a powerline pulse");

  plant.commodityCount[STUFF_HIVOLT] = 9000;
  {
    std::istringstream draws("LINCITY-SIMULATION-TRACE 1\nC 0\nZ\n");
    SimulationTrace trace(draws);
    feeder.trade();
    trace.finish();
  }
  require(feeder.anim_counter == POWER_MODULUS && relay.anim_counter == 0,
          "powered feeder did not start a pulse before its relay");
  feeder.animate(100);
  require(feeder.flashing, "the feeder pulse was not visible");
  feeder.trade();
  require(relay.anim_counter == POWER_MODULUS, "the pulse did not reach the neighboring powerline");
  relay.animate(100);
  feeder.trade();
  require(relay.flashing && relay.anim_counter == POWER_MODULUS - 1,
          "a pulse already in progress was restarted");
  require(relay.commodityCount[STUFF_HIVOLT] > 0 &&
          plant.commodityCount[STUFF_HIVOLT] + feeder.commodityCount[STUFF_HIVOLT] +
            relay.commodityCount[STUFF_HIVOLT] == 9000,
          "powerline trade created or lost electricity before the daily power loss tick");
}

void verifyRegistryDiagnostics() {
  World world(32);
  auto duplicate = blacksmithConstructionGroup;
  duplicate.resourceID = "UnregisteredSmithy";
  const auto& animation = *ResourceGroup::resMap.at("Blacksmith");
  std::ostringstream output;
  {
    struct RestoreOutput {
      std::streambuf* original;
      ~RestoreOutput() { std::cout.rdbuf(original); }
    } restore{std::cout.rdbuf(output.rdbuf())};
    ConstructionGroup::addConstructionGroup(&duplicate);
    ConstructionGroup::addResourceID("Blacksmith", &duplicate);
    ConstructionGroup::printGroups();
  }
  require(ConstructionGroup::getConstructionGroup(GROUP_BLACKSMITH) ==
          &blacksmithConstructionGroup, "duplicate group replaced the registered smithy");
  require(duplicate.resourceID == "UnregisteredSmithy", "duplicate resource was assigned to a new group");
  require(ResourceGroup::resMap.at("Blacksmith") == &animation,
          "duplicate resource replaced the registered animation");
  require(output.str().find("rejecting") != std::string::npos,
          "registry did not diagnose the duplicate");
  require(output.str().find("group #" + std::to_string(GROUP_BLACKSMITH) +
                            ": " + blacksmithConstructionGroup.name) != std::string::npos,
          "registry listing omitted the existing smithy");

  auto& tile = *world.map(MapPoint(12, 12));
  tile.setTerrain(NUM_OF_GROUPS);
  require(tile.getTileConstructionGroup()->group == GROUP_DESERT,
          "unrecognized terrain group did not use the desert fallback");
  require(tile.getTileResourceGroup() == ResourceGroup::resMap.at("Desert"),
          "unrecognized terrain did not use the desert animation");
  require(std::string_view(commodityName(STUFF_COUNT)) == "Unknown",
          "invalid inventory identifier did not display as unknown");
}

void verifyModule(ConstructionGroup& group) {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  const MapPoint origin(12, 12);
  if(group.group == GROUP_TRACK_BRIDGE || group.group == GROUP_ROAD_BRIDGE ||
     group.group == GROUP_RAIL_BRIDGE)
    world.map(origin)->setTerrain(GROUP_WATER);
  group.placeItem(world, origin);
  auto* building = world.map(origin)->getConstruction();
  require(building && building->getConstructionGroup() == &group, "wrong building placed");
  for(int y = 0; y < group.size; ++y) {
    for(int x = 0; x < group.size; ++x)
      require(world.map(origin.e(x).s(y))->getReportingConstruction() == building,
              "incomplete footprint");
  }
  for(unsigned long frame = 1; frame <= 3; ++frame) {
    world.do_animate(frame * 1000);
    world.do_time_step();
  }
  require(world.total_time == 3, "world failed to tick");
  if(world.map(origin)->getConstruction() != building) return;
  for(bool production : {false, true}) {
    Report report;
    building->report(report, production);
    require(report.rows > 0, "empty building report");
  }
}
}

int main() {
  std::srand(724);
  BasicUrbg::get().seed(724);
  World registration(32);
  int modules = 0;
  for(unsigned short id = 0; id < NUM_OF_GROUPS; ++id) {
    auto* group = ConstructionGroup::getConstructionGroup(id);
    if(!group) continue;
    try {
      verifyModule(*group);
      ++modules;
    } catch(const std::exception& error) {
      std::cerr << group->name << ": " << error.what() << '\n';
      return 1;
    }
  }
  if(modules != 43) {
    std::cerr << "Expected 43 building modules, exercised " << modules << '\n';
    return 1;
  }
  try {
    verifyBlacksmith();
    verifyRegistryDiagnostics();
    verifyMissingAnimationResource();
    verifyPlacementPreflight();
    verifyPlacementFactoryFailures();
    verifyEvacuationFit();
    verifyTradeWithNoReceivingCapacity();
    verifyTradeTaxesAndOutgoingCommuter();
    verifyMarketInventoryIsUntaxed();
    verifyTransportExchangeNearCapacity();
    verifyOldOverflowCannotSupplyEvacuatingNeighbor();
    verifyInventoryAndProductionReport();
    verifyTransportReport();
    verifyResidenceFootprints();
    verifyDetachedConstructionLifetime();
    verifyPowerCableLimits();
    verifyPowerlineRelay();
    verifyMill(STUFF_COAL, COAL_USED_BY_MILL);
    verifyMill(STUFF_LOVOLT, COAL_USED_BY_MILL * MILL_POWER_PER_COAL);
    verifySolarPower(0, POWERS_SOLAR_OUTPUT);
    verifySolarPower(MAX_TECH_LEVEL, 2 * POWERS_SOLAR_OUTPUT);
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
  std::cout << modules << " building modules exercised\n";
}
