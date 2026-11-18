#include "lincity/modules/cricket.hpp"
#include "lincity/modules/firestation.hpp"
#include "lincity/modules/health_centre.hpp"
#include "lincity/modules/market.hpp"
#include "lincity/modules/monument.hpp"
#include "lincity/modules/organic_farm.hpp"
#include "lincity/modules/port.hpp"
#include "lincity/modules/rocket_pad.hpp"
#include "lincity/modules/tip.hpp"
#include "lincity/world.hpp"
#include "util/simulation_trace.hpp"

#include <libxml++/exceptions/parse_error.h>

#ifdef LINCITY_TEST_XML_FAULTS
#include <libxml/xmlIO.h>
#include <libxml/xmlreader.h>
#include <libxml/xmlwriter.h>
#include <zlib.h>
#endif

#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <system_error>

#ifdef LINCITY_TEST_XML_FAULTS
namespace {
enum class XmlFactory { OutputBuffer, Writer, Reader };

struct FactoryFailure;
thread_local FactoryFailure* activeFailure = nullptr;

struct FactoryFailure {
  explicit FactoryFailure(XmlFactory factory) : factory(factory) { activeFailure = this; }
  ~FactoryFailure() { activeFailure = nullptr; }

  XmlFactory factory;
  bool triggered = false;
  int gzipCloses = 0;
};

bool shouldFail(XmlFactory factory) {
  if(!activeFailure || activeFailure->factory != factory || activeFailure->triggered) return false;
  activeFailure->triggered = true;
  return true;
}
}

extern "C" {
decltype(xmlOutputBufferCreateIO) __real_xmlOutputBufferCreateIO;
decltype(xmlNewTextWriter) __real_xmlNewTextWriter;
decltype(xmlReaderForIO) __real_xmlReaderForIO;
decltype(gzclose) __real_gzclose;

xmlOutputBufferPtr __wrap_xmlOutputBufferCreateIO(xmlOutputWriteCallback write,
  xmlOutputCloseCallback close, void* context, xmlCharEncodingHandlerPtr encoder
) {
  if(shouldFail(XmlFactory::OutputBuffer)) return nullptr;
  return __real_xmlOutputBufferCreateIO(write, close, context, encoder);
}

xmlTextWriterPtr __wrap_xmlNewTextWriter(xmlOutputBufferPtr buffer) {
  if(shouldFail(XmlFactory::Writer)) return nullptr;
  return __real_xmlNewTextWriter(buffer);
}

xmlTextReaderPtr __wrap_xmlReaderForIO(xmlInputReadCallback read,
  xmlInputCloseCallback close, void* context, const char* url, const char* encoding, int options
) {
  if(shouldFail(XmlFactory::Reader)) {
    // libxml may close the input even when creating the reader fails.
    if(close) close(context);
    return nullptr;
  }
  return __real_xmlReaderForIO(read, close, context, url, encoding, options);
}

int __wrap_gzclose(gzFile file) {
  // Observe an accidental second close without calling zlib on a freed stream.
  if(activeFailure && ++activeFailure->gzipCloses > 1) return Z_STREAM_ERROR;
  return __real_gzclose(file);
}
}
#endif

namespace {
namespace fs = std::filesystem;

#define CHECK(expression) do { if(!(expression)) throw std::runtime_error( \
  std::string(__func__) + " line " + std::to_string(__LINE__) + ": " #expression); } while(false)

template<typename Building>
const Building& buildingAt(const World& world, MapPoint point) {
  const auto* building = dynamic_cast<const Building*>(world.map(point)->getConstruction());
  CHECK(building);
  return *building;
}

void marketDirections(const fs::path& saved) {
  World world(32);
  const MapPoint point(12, 12);
  marketConstructionGroup.placeItem(world, point);
  CHECK(world.setMarketTrade(point, STUFF_FOOD, true, false));
  CHECK(world.setMarketTrade(point, STUFF_GOODS, false, true));
  world.save(saved);

  auto restored = World::load(saved);
  const auto& rules = buildingAt<Market>(*restored, point).commodityRuleCount;
  CHECK(rules[STUFF_FOOD].take && !rules[STUFF_FOOD].give);
  CHECK(!rules[STUFF_GOODS].take && rules[STUFF_GOODS].give);
}

void saveAfterBulldozing(const fs::path& saved) {
  World world(32);
  const MapPoint removed(10, 10);
  const MapPoint remaining(20, 20);
  marketConstructionGroup.placeItem(world, removed);
  marketConstructionGroup.placeItem(world, remaining);
  CHECK(world.setMarketTrade(remaining, STUFF_FOOD, false, true));
  world.bulldozeArea(removed.se());
  CHECK(world.map(removed)->getConstruction() == nullptr);

  // The user can save before the next tick cleans up the demolished market.
  CHECK(world.map.constructionCount() == 2);
  world.save(saved);
  world.do_time_step();

  const auto restored = World::load(saved);
  CHECK(restored->map.constructionCount() == 1);
  CHECK(restored->map(removed)->getConstruction() == nullptr);
  CHECK(restored->map(removed.se())->getReportingConstruction() == nullptr);
  const auto& rule = buildingAt<Market>(*restored, remaining).commodityRuleCount[STUFF_FOOD];
  CHECK(!rule.take && rule.give);
}

void checkMigratedWorld(const World& world) {
  CHECK(world.given_scene == "Legacy accounting and history" && world.total_time == 309);
  CHECK(world.map.len() == 32 && world.map.recentPoint == MapPoint(12, 12));
  CHECK(world.tech_level == 7500 && world.stats.tech_level == 7500);
  const auto& expenses = world.stats.expenses;
  CHECK(world.total_money == 1501);
  CHECK(expenses.import.stat == 58 && expenses.import.acc == 70);
  CHECK(expenses.unemployment.acc == 21 && expenses.transport.acc == 13);
  CHECK(expenses.health.acc == 18 && expenses.firestation.acc == 24);
  CHECK(expenses.windmill.acc == 13 && expenses.university.acc == 9 && expenses.recycle.acc == 6);
  CHECK(expenses.deaths.acc == 3 && expenses.rockets.acc == 11);
  CHECK(expenses.school.acc == 4 && expenses.cricket.acc == 7);
  CHECK(world.stats.income.income_tax.stat == 37 && world.taxable.labor == 91);
  const auto& population = world.stats.population;
  CHECK(population.housed_m.acc == 72121 && population.housing_m.acc == 91800);
  CHECK(population.starving_m.acc == 1034 && population.unemployed_m.acc == 1037);
  CHECK(population.starve_deaths_t == 6 && population.starve_deaths_history == 2.75f);
  CHECK(population.pollution_deaths_t == 8 && population.pollution_deaths_history == 3.125f);
  CHECK(population.unemployed_days_t == 2417);
  const auto& sustainability = world.stats.sustainability;
  CHECK(!sustainability.mining_flag && sustainability.mining_years == 4);
  CHECK(sustainability.trade_flag && sustainability.trade_years == 7);
  CHECK(!world.tradeRule[STUFF_FOOD].take && world.tradeRule[STUFF_FOOD].give);
  CHECK(world.tradeRule[STUFF_GOODS].take && !world.tradeRule[STUFF_GOODS].give);

  const auto& history = world.stats.history;
  CHECK(history.pop.at(0) == 901 && history.pop.at(2) == 701);
  CHECK(history.starve.at(1) == 23 && history.nojobs.at(2) == 27 && history.ppool.at(1) == 109);
  CHECK(history.tech.at(0) == 7500 && history.tech.at(1) == 7250);
  CHECK(history.tech.at(2) == 7000 && history.tech.at(3) == 7450);
  CHECK(history.money.at(0) == -326 && history.money.at(1) == -281);
  CHECK(history.pollution.at(0) == 275 && history.pollution.at(1) == 256);
  const auto& food = history.inventory[STUFF_FOOD];
  CHECK(food.at(0).amount == 660 && food.at(0).capacity == 1000);
  CHECK(food.at(1).amount == 620 && food.at(1).capacity == 1000);
  CHECK(history.tenants.at(0).amount == 910 && history.tenants.at(0).capacity == 1000);
  CHECK(history.tenants.at(1).amount == 890 && history.tenants.at(1).capacity == 1000);

  const auto& tile = *world.map(MapPoint(6, 8));
  CHECK(tile.group == GROUP_WATER && tile.is_river() && tile.ground.altitude == 17);
  CHECK((tile.flags & (FLAG_HAS_UNDERGROUND_WATER | FLAG_MARKET_COVER | FLAG_MARKET_COVER_CHECK))
        == (FLAG_HAS_UNDERGROUND_WATER | FLAG_MARKET_COVER | FLAG_MARKET_COVER_CHECK));
  const auto& lake = *world.map(MapPoint(7, 8));
  CHECK(lake.is_lake());
  constexpr auto services = FLAG_FIRE_COVER | FLAG_FIRE_COVER_CHECK |
    FLAG_HEALTH_COVER | FLAG_HEALTH_COVER_CHECK | FLAG_CRICKET_COVER | FLAG_CRICKET_COVER_CHECK;
  CHECK((lake.flags & services) == services);
  const auto& monument = buildingAt<Monument>(world, MapPoint(12, 12));
  CHECK(monument.tech_made == 3 && monument.tail_off == 2);
  CHECK(monument.completion == 18 && monument.labor_consumed == 63000);
}

void migrate2130(const fs::path& fixtures, const fs::path& saved) {
  auto world = World::load(fixtures / "migration-2130.xml");
  checkMigratedWorld(*world);
  world->save(saved);
  auto restored = World::load(saved);
  checkMigratedWorld(*restored);

  for(auto* city : {world.get(), restored.get()}) {
    city->do_time_step();
    const auto& farm = buildingAt<Organic_farm>(*city, MapPoint(20, 12));
    const auto& stock = farm.commodityCount;
    // One tile has groundwater, and 350 stored water irrigates seven more.
    // The eight irrigated tiles yield 330 food with the rebuilt tech bonus.
    CHECK(stock[STUFF_FOOD] == 350 && stock[STUFF_LABOR] == 17);
    CHECK(stock[STUFF_LOVOLT] == 25 && stock[STUFF_WATER] == 0 && stock[STUFF_WASTE] == 14);
  }
}

void checkStatistics2140(const World& world) {
  CHECK(world.given_scene == "Frühjahr in der Altstadt" && world.total_time == 1249);
  CHECK(world.total_money == -492 && world.stats.total_money == -492 && world.people_pool == 12);
  CHECK(world.tech_level == 96800 && world.stats.highest_tech_level == 98900);
  CHECK(world.rockets_launched == 9 && world.rockets_launched_success == 6);
  CHECK(world.gameEnd && world.coal_survey_done && world.stats.total_pollution == 324);
  CHECK(world.money_rates.income_tax == 17 && world.money_rates.transport_cost == 21);
  CHECK(world.taxable.labor == 312 && world.taxable.trade_ex == 74);
  CHECK(!world.tradeRule[STUFF_FOOD].take && world.tradeRule[STUFF_FOOD].give);
  const auto& population = world.stats.population;
  CHECK(population.population_m.stat == 6700 && population.population_m.acc == 3500);
  CHECK(population.starving_m.stat == 40 && population.starving_m.acc == 17);
  CHECK(population.births_t == 42 && population.max_pop_ever == 164);
  CHECK(population.starve_deaths_t == 9 && population.starve_deaths_history == 2.625f);
  CHECK(world.stats.income.goods_tax.stat == -153 && world.stats.income.goods_tax.acc == -94);
  CHECK(world.stats.expenses.coalSurvey.stat == 100 && world.stats.expenses.health.acc == 69);
  const auto& sustainability = world.stats.sustainability;
  CHECK(!sustainability.mining_flag && sustainability.trade_flag);
  CHECK(sustainability.old_money == -512 && sustainability.fire_years == 7);

  const auto& history = world.stats.history;
  CHECK(history.births.at(0) == 10 && history.births.at(1) == 0 && history.births.at(3) == 8);
  const auto& food = history.inventory[STUFF_FOOD];
  CHECK(food.at(0).amount == 3100 && food.at(0).capacity == 5500);
  CHECK(food.at(1).amount == 1200 && food.at(1).capacity == 5000);

  CHECK(world.map.recentPoint == MapPoint(5, 17));
  CHECK(world.map.alt_min == -12 && world.map.alt_max == 36 && world.map.alt_step == 4);
  const auto& lake = *world.map(MapPoint(6, 8));
  CHECK(lake.is_lake() && (lake.flags & FLAG_HAS_UNDERGROUND_WATER));
  CHECK(!(lake.flags & VOLATILE_FLAGS));
  CHECK(lake.pollution == 61 && lake.ore_reserve == 170 && lake.coal_reserve == 240);
  const auto& ground = lake.ground;
  CHECK(ground.altitude == -12 && ground.ecotable == 97);
  CHECK(ground.wastes == 3 && ground.pollution == 26);
  CHECK(ground.water_alt == 5 && ground.water_pol == 12 && ground.water_wast == 8);
}

void statistics2140(const fs::path& fixtures, const fs::path& saved) {
  const auto world = World::load(fixtures / "statistics-2140.xml");
  checkStatistics2140(*world);
  world->save(saved);
  checkStatistics2140(*World::load(saved));
}

void extendedPopulationHistory(const fs::path& saved) {
  World world(32);
  world.stats.ensurePopulationHistory(240);
  auto& history = world.stats.history;
  history.pop.at(190) = 2400;
  history.starve.at(211) = 52;
  history.nojobs.at(224) = 130;
  history.ppool.at(239) = 142;
  world.save(saved);

  auto restored = World::load(saved);
  const auto& restoredHistory = restored->stats.history;
  CHECK(restoredHistory.pop.size() == 240 && restoredHistory.starve.size() == 240);
  CHECK(restoredHistory.nojobs.size() == 240 && restoredHistory.ppool.size() == 240);
  CHECK(restoredHistory.pop.at(190) == 2400 && restoredHistory.starve.at(211) == 52);
  CHECK(restoredHistory.nojobs.at(224) == 130 && restoredHistory.ppool.at(239) == 142);
}

const MapPoint readyRocket(13, 5);

void checkSpecializedBuildings(const World& world) {
  CHECK(world.map.constructionCount() == 8);
  const auto& building = buildingAt<RocketPad>(world, MapPoint(5, 5));
  CHECK(building.stage == RocketPad::BUILDING && building.steps == 450 && building.tech == 82000);
  CHECK(building.commodityCount[STUFF_LABOR] == 300);
  const auto& ready = buildingAt<RocketPad>(world, readyRocket);
  CHECK(ready.stage == RocketPad::AWAITING && ready.steps == 5000 && ready.tech == 78000);
  const auto& done = buildingAt<RocketPad>(world, MapPoint(21, 5));
  CHECK(done.stage == RocketPad::DONE && done.steps == 5000 && done.tech == 70000);

  const auto& port = buildingAt<Port>(world, MapPoint(12, 14));
  CHECK(port.tech_made == 9 && port.commodityCount[STUFF_GOODS] == 700);
  CHECK(port.commodityRuleCount[STUFF_FOOD].take && !port.commodityRuleCount[STUFF_FOOD].give);
  CHECK(!port.commodityRuleCount[STUFF_GOODS].take && port.commodityRuleCount[STUFF_GOODS].give);
  const auto& tip = buildingAt<Tip>(world, MapPoint(22, 13));
  CHECK(tip.total_waste == 14000 && tip.degration_days == 4);

  const auto& health = buildingAt<HealthCentre>(world, MapPoint(5, 22));
  CHECK(health.active && health.daycount == 8 && health.covercount == 31);
  const auto& firestation = buildingAt<FireStation>(world, MapPoint(13, 22));
  CHECK(!firestation.active && firestation.daycount == 4 && firestation.covercount == 29);
  const auto& cricket = buildingAt<Cricket>(world, MapPoint(21, 22));
  CHECK(cricket.active && cricket.daycount == 14 && cricket.covercount == 27);
}

void specializedBuildings(const fs::path& fixtures, const fs::path& saved) {
  auto world = World::load(fixtures / "specialized-buildings-2140.xml");
  checkSpecializedBuildings(*world);
  // Saving during a launch preserves the ready rocket so the player can retry.
  CHECK(world->launchRocket(readyRocket));
  CHECK(buildingAt<RocketPad>(*world, readyRocket).stage == RocketPad::LAUNCHING);
  world->save(saved);
  auto restored = World::load(saved);
  checkSpecializedBuildings(*restored);
  CHECK(restored->launchRocket(readyRocket));
}

void writeXml(const fs::path& path, std::string_view xml) {
  std::ofstream output(path, std::ios::binary | std::ios::trunc);
  output.exceptions(std::ios::failbit | std::ios::badbit);
  output << xml;
  output.close();
}

template<typename Error = std::runtime_error, typename Operation>
void expectFailure(Operation operation, std::string_view diagnostic) {
  try {
    operation();
  } catch(const Error& error) {
    if(std::string_view(error.what()).find(diagnostic) != std::string_view::npos) return;
    throw;
  }
  throw std::runtime_error("Expected rejection: " + std::string(diagnostic));
}

void rejectedConstructions(const fs::path& saved) {
  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <map size="32">
        <Construction group="6" map-x="5" map-y="5"/>
        <Construction group="33" map-x="20" map-y="20"/>
        <Construction group="26" map-x="13" map-y="13">
          <total_tech_made>unknown</total_tech_made>
        </Construction>
      </map>
    </lc-game>
  )xml");
  expectFailure([&] { World::load(saved); }, "failed to parse XML value");

  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <map size="32">
        <Construction group="23" map-x="12" map-y="12">
          <stage>unsupported</stage>
        </Construction>
      </map>
    </lc-game>
  )xml");
  expectFailure([&] { World::load(saved); }, "unknown rocket stage");

  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <map size="32">
        <Construction group="6" map-x="5" map-y="5"/>
        <Construction group="26" map-x="31" map-y="13"/>
        <Construction group="33" map-x="20" map-y="20"/>
      </map>
    </lc-game>
  )xml");
  expectFailure([&] { World::load(saved); }, "cannot place a Construction outside the map");

  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <map size="32">
        <Construction group="6" map-x="5" map-y="5"/>
        <Construction group="26" map-x="13" map-y="13"/>
        <Construction group="33" map-x="20" map-y="20"/>
      </map>
    </lc-game>
  )xml");
  {
    const fs::path overlapping = saved.string() + ".overlap";
    writeXml(overlapping, R"xml(
      <lc-game ldsv-version="2140">
        <map size="32">
          <Construction group="6" map-x="5" map-y="5"/>
          <Construction group="26" map-x="6" map-y="6"/>
        </map>
      </lc-game>
    )xml");
    expectFailure<std::logic_error>([&] { World::load(overlapping); }, "space occupied");
    fs::remove(overlapping);
  }
  {
    // The second placement has reached the map by the time replay checks it.
    std::istringstream events("LINCITY-SIMULATION-TRACE 1\nP 1 5 5 6\nP 2 14 13 26\nZ\n");
    SimulationTrace replay(events);
    expectFailure([&] { World::load(saved); }, "construction placement differs");
  }
  CHECK(World::load(saved)->map.constructionCount() == 3);
}

#ifdef LINCITY_TEST_XML_FAULTS
void factoryErrors(const fs::path& fixtures, const fs::path& saved) {
  const World world(32);
  struct Case { XmlFactory factory; const char* diagnostic; };
  for(const auto& test : {
    Case{XmlFactory::OutputBuffer, "failed to create XML output buffer"},
    Case{XmlFactory::Writer, "failed to create XML text writer"},
    Case{XmlFactory::Reader, "failed to initialize XML parser"}
  }) {
    FactoryFailure failure(test.factory);
    expectFailure([&] {
      if(test.factory == XmlFactory::Reader) World::load(fixtures / "statistics-2140.xml");
      else world.save(saved);
    }, test.diagnostic);
    CHECK(failure.triggered);
    CHECK(failure.gzipCloses == 1);
  }

  world.save(saved);
  CHECK(World::load(saved)->map.len() == 32);
}
#endif

void rejectEmptySave(const fs::path& saved) {
  writeXml(saved, "");
  try {
    World::load(saved);
  } catch(const xmlpp::parse_error&) {
    return;
  } catch(const std::runtime_error& error) {
    CHECK(std::string_view(error.what()).find("save file is empty") != std::string_view::npos);
    return;
  }
  throw std::runtime_error("Expected rejection of an empty city save");
}

void rejectTruncatedCompression(const fs::path& saved) {
  World world(32);
  world.given_scene = "Truncated compressed save";
  world.save(saved);
  auto complete = World::load(saved);
  CHECK(complete->given_scene == world.given_scene);

  // All XML is still present, but the gzip stream is missing its size footer.
  const auto size = fs::file_size(saved);
  CHECK(size > 8);
  fs::resize_file(saved, size - 4);
  expectFailure([&] { World::load(saved); }, "failed to read compressed save file");
}

void parserErrors(const fs::path& saved) {
  rejectEmptySave(saved);
  writeXml(saved, "<SaveGame/>");
  expectFailure([&] { World::load(saved); }, "format that is too old");
  writeXml(saved, "<lc-game ldsv-version='latest'/>");
  expectFailure([&] { World::load(saved); }, "failed to parse XML value");
  // A regular file cannot be the parent of a save path.
  const auto unavailable = saved / "unavailable.sav.gz";
  expectFailure([&] { World::load(unavailable); }, "failed to open file");
  const World world(32);
  expectFailure([&] { world.save(unavailable); }, "failed to open file");

  writeXml(saved, "<lc-game ldsv-version='2140'><map size='32'>"
                  "<recentPoint><y>7</y></recentPoint></map></lc-game>");
  expectFailure([&] { World::load(saved); }, "missing XML element <x>");
  writeXml(saved, "<lc-game ldsv-version='2140'><map size='32'>"
                  "<recentPoint><x>7</x></recentPoint></map></lc-game>");
  expectFailure([&] { World::load(saved); }, "missing XML element <y>");
  writeXml(saved, "<lc-game ldsv-version='2140'><map size='medium'/></lc-game>");
  expectFailure([&] { World::load(saved); }, "failed to parse XML value");
  writeXml(saved, "<lc-game ldsv-version='2140'><globals><game_end>maybe</game_end>"
                  "</globals></lc-game>");
  expectFailure([&] { World::load(saved); }, "failed to parse bool");

  for(const char* legacyField : {
    "<monthgraph_pop>90 missing 70</monthgraph_pop>",
    "<pbar id='1'><data>7000 missing</data></pbar>",
    "<pbar id='3'><data>660 missing</data></pbar>"
  }) {
    writeXml(saved, std::string("<lc-game ldsv-version='2130'><globals>") + legacyField
                   + "</globals></lc-game>");
    try {
      World::load(saved);
    } catch(const std::system_error& error) {
      CHECK(error.code() == std::errc::invalid_argument);
      continue;
    }
    throw std::runtime_error(std::string("Expected rejection of legacy field: ") + legacyField);
  }
}

#ifdef __linux__
void rejectFailedWrite() {
  const fs::path fullDevice("/dev/full");
  std::error_code statusError;
  if(!fs::is_character_file(fs::symlink_status(fullDevice, statusError))) return;

  const World world(32);
  expectFailure([&] { world.save(fullDevice); }, "failed to write save file");
}
#endif

#ifdef NDEBUG
void unknownFields(const fs::path& saved) {
  writeXml(saved, "<settings/>");
  expectFailure([&] { World::load(saved); }, "failed to find <lc-game>");

  // Release builds skip unknown fields and keep reading the known siblings.
  writeXml(saved, R"xml(
    <lc-game ldsv-version="2140">
      <preview><total_money>375</total_money></preview>
      <globals>
        <money_rates>
          <regional_tax_rate>13</regional_tax_rate>
          <income_tax_rate>7</income_tax_rate>
        </money_rates>
        <taxable>
          <regional_trade>4</regional_trade>
          <labor>81</labor>
        </taxable>
        <tradeRules>
          <import_fish_enable>1</import_fish_enable>
          <import_food_enable>0</import_food_enable>
        </tradeRules>
        <stats>
          <population>
            <commuters_lm>17</commuters_lm>
            <housed_lm>74</housed_lm>
          </population>
          <income>
            <regional_tax_ly>-85</regional_tax_ly>
            <goods_tax_ly>-22</goods_tax_ly>
          </income>
          <expenses>
            <transit_ytd>18</transit_ytd>
            <transport_ytd>9</transport_ytd>
          </expenses>
          <sustainability>
            <water_years>2</water_years>
            <fire_years>3</fire_years>
          </sustainability>
          <regional/>
          <history>
            <commuters/>
            <money>
              <metadata>123</metadata>
              <index_latest>43</index_latest>
              <index_12>99</index_12>
              <index_1>-37</index_1>
            </money>
            <food>
              <index_0><label>Winter</label><amount>45</amount><capacity>100</capacity></index_0>
            </food>
          </history>
        </stats>
        <camera/>
        <total_money>81</total_money>
      </globals>
      <map size="32">
        <extension/>
        <MapTile group="0" map-x="4" map-y="4">
          <soil_type>clay</soil_type>
          <altitude>19</altitude>
        </MapTile>
        <recentPoint><x>6</x><zoom>2</zoom><y>7</y></recentPoint>
      </map>
    </lc-game>
  )xml");
  auto world = World::load(saved);
  CHECK(world->total_money == 81 && world->map.recentPoint == MapPoint(6, 7));
  CHECK(world->money_rates.income_tax == 7 && world->taxable.labor == 81);
  CHECK(!world->tradeRule[STUFF_FOOD].take);
  CHECK(world->stats.population.housed_m.stat == 74 && world->stats.income.goods_tax.stat == -22);
  CHECK(world->stats.expenses.transport.acc == 9 && world->stats.sustainability.fire_years == 3);
  CHECK(world->map(MapPoint(4, 4))->ground.altitude == 19);
  const auto& money = world->stats.history.money;
  CHECK(money.size() == 12 && money.at(0) == 0 && money.at(1) == -37);
  const auto& food = world->stats.history.inventory[STUFF_FOOD].at(0);
  CHECK(food.amount == 45 && food.capacity == 100);
}

void unknownLegacyFields(const fs::path& saved) {
  writeXml(saved, R"xml(
    <lc-game ldsv-version="2130">
      <globals>
        <world_side_len>32</world_side_len>
        <pbar id="2">
          <label>Money</label>
          <diff>30</diff>
          <data>650</data>
        </pbar>
        <pbar id="3">
          <label>Food</label>
          <diff>20</diff>
          <data>320</data>
        </pbar>
        <editor_note>From the old campaign</editor_note>
        <total_money>650</total_money>
      </globals>
      <map/>
    </lc-game>
  )xml");
  const auto world = World::load(saved);
  CHECK(world->total_money == 650 && world->map.len() == 32);
  const auto& history = world->stats.history;
  CHECK(history.money.at(0) == 650 && history.money.at(1) == 620);
  const auto& food = history.inventory[STUFF_FOOD];
  CHECK(food.at(0).amount == 320 && food.at(0).capacity == 1000);
  CHECK(food.at(1).amount == 300 && food.at(1).capacity == 1000);
}
#endif
}

int main(int argc, char** argv) {
  if(argc != 3) return 2;
  const fs::path fixtures = argv[1];
  const fs::path saved = argv[2];
  try {
    marketDirections(saved);
    saveAfterBulldozing(saved);
    migrate2130(fixtures, saved);
    statistics2140(fixtures, saved);
    extendedPopulationHistory(saved);
    specializedBuildings(fixtures, saved);
    parserErrors(saved);
    rejectTruncatedCompression(saved);
    rejectedConstructions(saved);
#ifdef LINCITY_TEST_XML_FAULTS
    factoryErrors(fixtures, saved);
#endif
#ifdef __linux__
    rejectFailedWrite();
#endif
#ifdef NDEBUG
    unknownFields(saved);
    unknownLegacyFields(saved);
#endif
    fs::remove(saved);
  } catch(const std::exception& ex) {
    std::cerr << ex.what() << '\n';
    return 1;
  }
}
