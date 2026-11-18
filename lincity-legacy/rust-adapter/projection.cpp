#include "projection.hpp"
#include "lincity/ConstructionReport.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/world.hpp"
#include "rustcity-legacy/src/view.rs.h"
#include <algorithm>
#include <stdexcept>
#include <unordered_map>
#include <vector>

namespace view = rustcity::view;
struct RustViewProjection::State {
  std::unordered_map<int, std::uint64_t> buildings;
  struct VehicleSprite {
    MapTile *tile;
    std::list<ExtraFrame>::iterator frame;
  };
  std::vector<VehicleSprite> vehicles;
};
RustViewProjection::RustViewProjection() : state(std::make_unique<State>()) {}
RustViewProjection::~RustViewProjection() = default;

namespace {
void copyPeriod(Stat<> &to, const view::Period &from) {
  to.stat = from.previous;
  to.acc = from.current;
}
void copyInventory(std::deque<Stats::Inventory<>> &to,
                   const rust::Vec<view::Inventory> &from) {
  to.resize(from.size());
  for (std::size_t i = 0; i < from.size(); ++i)
    to[i] = {from[i].amount, from[i].capacity};
}

static void syncStatistics(World &w, const rustcity::view::Statistics &s) {
  w.total_time = s.day;
  w.total_money = s.balance;
  w.tech_level = s.tech;
  w.people_pool = s.people_pool;
  w.rockets_launched = s.launched;
  w.rockets_launched_success = s.successes;
  w.gameEnd = s.game_end;
  w.coal_survey_done = s.survey;
  w.given_scene = std::string(s.name);
  w.map.recentPoint = {s.focus_x, s.focus_y};
  auto &t = w.stats;
  t.highest_tech_level = s.highest_tech_level;
  t.total_pollution = s.total_pollution;
  t.tech_level = s.stats_tech;
  t.total_money = s.stats_money;
  std::copy(s.group_count.begin(), s.group_count.end(), t.groupCount.begin());
  for (unsigned i = 0; i < STUFF_COUNT; ++i) {
    w.tradeRule[i].take = s.foreign_buy[i];
    w.tradeRule[i].give = s.foreign_sell[i];
    copyPeriod(t.inventory[i].amount, s.inventory[i].amount);
    copyPeriod(t.inventory[i].capacity, s.inventory[i].capacity);
    copyInventory(t.history.inventory[i], s.history.inventory[i].samples);
  }
  copyPeriod(t.population.population_m, s.population.population_m);
  copyPeriod(t.population.housed_m, s.population.housed_m);
  copyPeriod(t.population.starving_m, s.population.starving_m);
  copyPeriod(t.population.unemployed_m, s.population.unemployed_m);
  copyPeriod(t.population.housing_m, s.population.housing_m);
  copyPeriod(t.population.births_m, s.population.births_m);
  copyPeriod(t.population.deaths_m, s.population.deaths_m);
  copyPeriod(t.population.unnat_deaths_m, s.population.unnat_deaths_m);
  t.population.evacuated_t = s.population.evacuated_t;
  t.population.births_t = s.population.births_t;
  t.population.pollution_deaths_t = s.population.pollution_deaths_t;
  t.population.starve_deaths_t = s.population.starve_deaths_t;
  t.population.unemployed_days_t = s.population.unemployed_days_t;
  t.population.max_pop_ever = s.population.max_pop_ever;
  t.population.unemployed_history = s.population.unemployed_history;
  t.population.pollution_deaths_history = s.population.pollution_deaths_history;
  t.population.starve_deaths_history = s.population.starve_deaths_history;
  copyPeriod(t.income.income_tax, s.income.income_tax);
  copyPeriod(t.income.coal_tax, s.income.coal_tax);
  copyPeriod(t.income.goods_tax, s.income.goods_tax);
  copyPeriod(t.income.export_tax, s.income.export_tax);
  copyPeriod(t.expenses.construction, s.expenses.construction);
  copyPeriod(t.expenses.coalSurvey, s.expenses.coal_survey);
  copyPeriod(t.expenses.import, s.expenses.import);
  copyPeriod(t.expenses.unemployment, s.expenses.unemployment);
  copyPeriod(t.expenses.transport, s.expenses.transport);
  copyPeriod(t.expenses.windmill, s.expenses.windmill);
  copyPeriod(t.expenses.university, s.expenses.university);
  copyPeriod(t.expenses.recycle, s.expenses.recycle);
  copyPeriod(t.expenses.deaths, s.expenses.deaths);
  copyPeriod(t.expenses.health, s.expenses.health);
  copyPeriod(t.expenses.rockets, s.expenses.rockets);
  copyPeriod(t.expenses.school, s.expenses.school);
  copyPeriod(t.expenses.firestation, s.expenses.firestation);
  copyPeriod(t.expenses.cricket, s.expenses.cricket);
  copyPeriod(t.expenses.interest, s.expenses.interest);
  t.sustainability.mining_flag = s.sustainability.mining_flag;
  t.sustainability.trade_flag = s.sustainability.trade_flag;
  t.sustainability.sustainable = s.sustainability.sustainable;
  t.sustainability.mining_years = s.sustainability.mining_years;
  t.sustainability.trade_years = s.sustainability.trade_years;
  t.sustainability.old_money = s.sustainability.old_money;
  t.sustainability.money_years = s.sustainability.money_years;
  t.sustainability.old_population = s.sustainability.old_population;
  t.sustainability.population_years = s.sustainability.population_years;
  t.sustainability.old_tech = s.sustainability.old_tech;
  t.sustainability.tech_years = s.sustainability.tech_years;
  t.sustainability.fire_years = s.sustainability.fire_years;
  t.history.pop.assign(s.history.pop.begin(), s.history.pop.end());
  t.history.starve.assign(s.history.starve.begin(), s.history.starve.end());
  t.history.nojobs.assign(s.history.nojobs.begin(), s.history.nojobs.end());
  t.history.ppool.assign(s.history.ppool.begin(), s.history.ppool.end());
  t.history.tech.assign(s.history.tech.begin(), s.history.tech.end());
  t.history.money.assign(s.history.money.begin(), s.history.money.end());
  t.history.pollution.assign(s.history.pollution.begin(),
                             s.history.pollution.end());
  t.history.births.assign(s.history.births.begin(), s.history.births.end());
  t.history.deaths.assign(s.history.deaths.begin(), s.history.deaths.end());
  t.history.unnat_deaths.assign(s.history.unnat_deaths.begin(),
                                s.history.unnat_deaths.end());
  copyInventory(t.history.tenants, s.history.tenants);
}
void syncTerrain(World &w, const rust::Vec<view::Tile> &tiles) {
  auto dest = w.map.begin();
  int low = tiles[0].altitude, high = low;
  for (const auto &t : tiles) {
    auto &d = *dest++;
    d.group = t.ground;
    d.type = t.frame;
    d.pollution = t.pollution;
    d.ore_reserve = t.ore;
    d.coal_reserve = t.coal;
    d.ground.altitude = t.altitude;
    d.ground.water_alt = t.water_altitude;
    low = std::min(low, t.altitude);
    high = std::max(high, t.altitude);
    d.flags = (t.lake ? FLAG_IS_LAKE : 0) | (t.river ? FLAG_IS_RIVER : 0) |
              (t.groundwater ? FLAG_HAS_UNDERGROUND_WATER : 0) |
              (t.cable_ns ? FLAG_POWER_CABLES_0 : 0) |
              (t.cable_ew ? FLAG_POWER_CABLES_90 : 0) |
              (t.fire ? FLAG_FIRE_COVER : 0) |
              (t.health ? FLAG_HEALTH_COVER : 0) |
              (t.market ? FLAG_MARKET_COVER : 0) |
              (t.sports ? FLAG_CRICKET_COVER : 0);
  }
  w.map.alt_min = low;
  w.map.alt_max = high;
  w.map.alt_step = (high - low) / 10;
}

void copyBuilding(Construction &b, const view::Building &src) {
  b.commodityCount = src.stock;
  b.flags = (b.flags & ~(FLAG_EVACUATE | FLAG_NEVER_EVACUATE)) |
            (src.evacuating ? FLAG_EVACUATE : 0) |
            (src.never_evacuate ? FLAG_NEVER_EVACUATE : 0);
  if (auto *transport = dynamic_cast<Transport *>(&b))
    transport->trafficCount = src.traffic;
  if (auto *line = dynamic_cast<Powerline *>(&b))
    line->trafficCount = src.traffic;
  if (auto *residence = dynamic_cast<Residence *>(&b))
    residence->local_population = src.occupants;
  if (auto *fire = dynamic_cast<Fire *>(&b)) {
    fire->burning_days = src.burning;
    fire->smoking_days = src.smoking;
  }
  auto trade = [&](auto &local) {
    for (unsigned i = 0; i < STUFF_COUNT; ++i) {
      local[i].take = src.buy[i];
      local[i].give = src.sell[i];
    }
  };
  if (auto *market = dynamic_cast<Market *>(&b))
    trade(market->commodityRuleCount);
  if (auto *port = dynamic_cast<Port *>(&b))
    trade(port->commodityRuleCount);
}
void copySprites(World &w, const rust::Vec<view::Animation> &animations) {
  for (const auto &a : animations) {
    auto &tile = *w.map({a.x, a.y});
    auto *b = tile.getConstruction();
    if (!b || a.layers.empty())
      throw std::logic_error("unmapped Rust sprite");
    b->flags =
        (b->flags & ~FLAG_TRANSPARENT) | (a.transparent ? FLAG_TRANSPARENT : 0);
    if (auto *rocket = dynamic_cast<RocketPad *>(b)) {
      switch (a.rocket_stage) {
      case view::RocketPhase::Building:
        rocket->stage = RocketPad::BUILDING;
        break;
      case view::RocketPhase::Awaiting:
        rocket->stage = RocketPad::AWAITING;
        break;
      case view::RocketPhase::Launching:
        rocket->stage = RocketPad::LAUNCHING;
        break;
      case view::RocketPhase::Launch:
        rocket->stage = RocketPad::LAUNCH;
        break;
      case view::RocketPhase::Done:
        rocket->stage = RocketPad::DONE;
        break;
      }
    }
    // The construction owns the iterators for its optional overlays. Keep even
    // hidden nodes.
    size_t next = 0;
    tile.visitFrames([&](ExtraFrame &f) {
      if (f.resourceGroup && f.resourceGroup->is_vehicle)
        return;
      if (next >= a.layers.size()) {
        f.frame = -1;
        return;
      }
      const auto &l = a.layers[next++];
      f.resourceGroup = ResourceGroup::resMap.at(std::string(l.resource));
      f.frame = l.frame;
      f.move_x = l.dx;
      f.move_y = l.dy;
    });
    if (next != a.layers.size())
      throw std::logic_error("missing native building sprite layer");
  }
}
} // namespace

void syncRustView(World &w, const rustcity::Engine &engine, bool changed,
                  RustViewProjection &projection) {
  auto &state = *projection.state;
  if (changed) {
    const auto tiles = view::tiles(engine);
    const auto buildings = view::buildings(engine);
    const int side = w.map.len();
    if (tiles.size() != static_cast<std::size_t>(side) * side)
      throw std::logic_error("Rust city and view dimensions differ");
    std::unordered_map<int, std::uint64_t> current;
    std::vector<std::pair<MapPoint, std::unique_ptr<Construction>>> added;
    for (const auto &src : buildings) {
      const int i = src.x + src.y * side;
      current.emplace(i, src.id);
      auto old = state.buildings.find(i);
      if (old != state.buildings.end() && old->second == src.id)
        continue;
      auto *g = ConstructionGroup::getConstructionGroup(src.group);
      if (!g)
        throw std::logic_error("unknown Rust construction group");
      added.emplace_back(MapPoint(src.x, src.y), std::unique_ptr<Construction>(
                                                     g->createConstruction(w)));
    }
    for (const auto &[i, id] : state.buildings) {
      auto now = current.find(i);
      if (now == current.end() || now->second != id)
        w.map.discardConstruction(
            *w.map({i % side, i / side})->getConstruction());
    }
    // Native constructors use terrain to resolve road/bridge variants. Their
    // placement can change tile frames and deposits, which remain authoritative
    // in the Rust snapshot.
    for (auto &[p, building] : added) {
      auto &tile = *w.map(p);
      tile.group = tiles[p.x + p.y * side].ground;
      Construction::placeOwned(std::move(building), p);
    }
    state.buildings.swap(current);
    for (const auto &src : buildings)
      copyBuilding(*w.map({src.x, src.y})->getConstruction(), src);
    syncTerrain(w, tiles);
    syncStatistics(w, view::statistics(engine));
    for (int k = static_cast<int>(World::Updatable::POPULATION);
         k <= static_cast<int>(World::Updatable::SUSTAINABILITY); ++k)
      w.setUpdated(static_cast<World::Updatable>(k));
  }
  for (auto &car : state.vehicles)
    car.tile->killframe(car.frame);
  state.vehicles.clear();
  copySprites(w, view::sprites(engine));
  for (const auto &car : view::cars(engine)) {
    auto *tile = w.map({car.x, car.y});
    auto frame = tile->createframe();
    frame->resourceGroup = ResourceGroup::resMap.at("Bluecar");
    frame->frame = car.frame;
    frame->move_x = car.dx;
    frame->move_y = car.dy;
    state.vehicles.push_back({tile, frame});
  }
}

bool renderRustReport(const rustcity::Engine &engine, MapPoint point,
                      bool production, ConstructionReport &target) {
  const auto rows = view::report(engine, point.x, point.y, production);
  for (const auto &r : rows) {
    const std::string label(r.label), text(r.text);
    using K = view::RowKind;
    switch (r.kind) {
    case K::Blank:
      target.addBlank();
      break;
    case K::Label:
      target.addLabel(label);
      break;
    case K::Text:
      target.addLabeledText(label, text);
      break;
    case K::Integer:
      target.addInteger(label, r.value);
      break;
    case K::Decimal:
      target.addDecimal(label, r.number);
      break;
    case K::Percentage:
      target.addPercentage(label, r.number);
      break;
    case K::Quantity:
      target.addQuantity(label, r.value, r.capacity);
      break;
    case K::Prefixed:
      target.addPrefixedQuantity(text, label, r.value, r.capacity);
      break;
    }
  }
  return !rows.empty();
}
