// Godot owns this façade. Each instance owns an independent simulation.
#include <godot_cpp/classes/project_settings.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/translation_server.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/godot.hpp>
#include <godot_cpp/variant/packed_int32_array.hpp>
#include <godot_cpp/variant/packed_byte_array.hpp>
#include <godot_cpp/variant/packed_float64_array.hpp>
#include <godot_cpp/variant/packed_vector3_array.hpp>
#include <godot_cpp/variant/typed_array.hpp>

#include "DesktopWorld.hpp"
#include "lincity/ConstructionReport.hpp"
#include "lincity/sustainable.hpp"
#include "MapOverlay.hpp"
#include <godot_cpp/variant/packed_color_array.hpp>
#include "lincity/modules/all_modules.hpp"
#include "rustcity-legacy/src/view.rs.h"

#include <fmt/format.h>

#include <algorithm>
#include <array>
#include <cmath>
#include <limits>
#include <vector>

namespace {
using godot::Array;
using godot::ClassDB;
using godot::D_METHOD;
using godot::Dictionary;
using godot::PackedInt32Array;
using godot::String;
using godot::TypedArray;
using godot::Vector2i;

String text(const std::string& value) { return String::utf8(value.c_str()); }
String localized(const std::string& value) {
  return godot::TranslationServer::get_singleton()->translate(text(value));
}

template<class... Args>
String localized_format(const std::string& key, const Args&... args) {
  const auto format = localized(key).utf8();
  return text(fmt::format(fmt::runtime(format.get_data()), args...));
}

String message_text(const Message::ptr& message) {
  if(auto build = dynamic_message_cast<CannotBuildMessage>(message)) {
    const auto name = localized(build->getGroup().name).utf8();
    auto headline = localized_format(dynamic_message_cast<CannotBuildHereMessage>(message)
      ? "Cannot build a {} here." : "Cannot build a {}.", name.get_data());
    if(auto reason = build->getReason()) headline += " " + message_text(reason);
    return headline;
  }
  if(dynamic_message_cast<CannotBulldozeNonemptyTipMessage>(message))
    return localized("You cannot bulldoze this land fill because it is full of waste.");
  if(dynamic_message_cast<CannotBulldozeIncompleteMonumentMessage>(message))
    return localized("You cannot bulldoze this monument because it is still under construction.");
  if(auto bulldoze = dynamic_message_cast<CannotBulldozeThisMessage>(message)) {
    const auto name = localized(bulldoze->getGroup().name).utf8();
    return localized_format("Cannot bulldoze this {}.", name.get_data());
  }
  if(auto evacuate = dynamic_message_cast<CannotEvacuateThisMessage>(message)) {
    const auto name = localized(evacuate->getGroup().name).utf8();
    return localized_format("Cannot evacuate {}.", name.get_data());
  }
  if(dynamic_message_cast<CannotEvacuateNothingMessage>(message))
    return localized("There is nothing to evacuate here.");
  if(auto fire = dynamic_message_cast<FireStartedMessage>(message)) {
    const auto name = localized(fire->getGroup().name).utf8();
    return localized_format("A fire has started: {} {}.", name.get_data(), fire->getPoint());
  }
  if(auto money = dynamic_message_cast<OutOfMoneyMessage>(message))
    return localized(money->isOutOfCredit() ? "Out of Credit" : "Out of Money");
  if(dynamic_message_cast<NoOreMessage>(message))
    return localized("There are no ore reserves left here.");
  if(dynamic_message_cast<DesertHereMessage>(message))
    return localized("This space is desert and has no water.");
  if(dynamic_message_cast<PortRequiresRiverMessage>(message))
    return localized("A port requires a river along the east side.");
  if(dynamic_message_cast<NotEnoughStudentsMessage>(message))
    return localized("There are not enough students. Build more schools first.");
  if(dynamic_message_cast<OutsideMapMessage>(message))
    return localized("This space is outside the map.");
  if(auto tech = dynamic_message_cast<NotEnoughTechMessage>(message))
    return localized_format("Technology required: {:.1f}%. Current level: {:.1f}%.",
      tech->getRequiredTech() * 100.0 / MAX_TECH_LEVEL, tech->getCurrentTech() * 100.0 / MAX_TECH_LEVEL);
  if(dynamic_message_cast<SpaceOccupiedMessage>(message))
    return localized("This space is occupied.");
  if(dynamic_message_cast<SustainableEconomyMessage>(message))
    return localized("The economy is sustainable.");
  if(dynamic_message_cast<NoPeopleLeftMessage>(message))
    return localized("The city has lost all its inhabitants.");
  if(dynamic_message_cast<RocketReadyMessage>(message))
    return localized("A rocket is ready to launch.");
  if(auto rocket = dynamic_message_cast<RocketResultMessage>(message)) {
    using Result = RocketResultMessage::LaunchResult;
    switch(rocket->getResult()) {
      case Result::FAIL: return localized("The rocket crashed.");
      case Result::SUCCESS: return localized("The rocket launched successfully.");
      case Result::EVAC: return localized("1,000 people were evacuated.");
      case Result::EVAC_WIN: return localized("Everyone has been evacuated from the city.");
    }
  }
  return localized(message->str());
}
MapPoint point(Vector2i v) { return {v.x, v.y}; }
std::filesystem::path file_path(const String& path) {
  auto absolute = godot::ProjectSettings::get_singleton()->globalize_path(path);
  return std::filesystem::u8path(absolute.utf8().get_data());
}

template<class Command> String run(Command command, World* warning_world = nullptr) {
  try {
    command();
    return {};
  } catch(const Message::Exception& error) {
    if(warning_world && dynamic_message_cast<OutOfMoneyMessage>(error.getMessage()))
      warning_world->pushMessage(error.getMessage());
    return message_text(error.getMessage());
  } catch(const std::exception& error) {
    return localized(error.what());
  }
}

enum SpecialTool { water = -4, evacuate, demolish, inspect };

UserOperation operation(int id) {
  UserOperation op;
  switch(id) {
    case inspect: op.action = UserOperation::ACTION_QUERY; break;
    case demolish: op.action = UserOperation::ACTION_BULLDOZE; break;
    case evacuate: op.action = UserOperation::ACTION_EVACUATE; break;
    case water: op.action = UserOperation::ACTION_FLOOD; break;
    default:
      // The toolbar offers the base transports and windmill. The backend
      // resolves bridges and modern wind turbines from terrain and technology.
      if((id >= GROUP_POWER_LINE && id <= GROUP_CRICKET && id != GROUP_WATER)
          || (id >= GROUP_RESIDENCE_ML && id <= GROUP_WATERWELL)
          || id == GROUP_PARKPOND)
        op.constructionGroup = ConstructionGroup::getConstructionGroup(id);
      op.action = op.constructionGroup ? UserOperation::ACTION_BUILD : UserOperation::ACTION_UNKNOWN;
  }
  return op;
}

class Report final : public ConstructionReport {
public:
  String out;
  void row(const String& label, const String& value) { out += label + String("  ") + value + String("\n"); }
  void addBlank() override { out += "\n"; }
  void addLabel(const std::string& label) override { out += localized(label) + "\n"; }
  void addLabeledText(const std::string& label, const std::string& value) override {
    row(localized(label), localized(value));
  }
  void addInteger(const std::string& label, int value) override { row(localized(label), String::num_int64(value)); }
  void addDecimal(const std::string& label, double value) override { row(localized(label), text(fmt::format("{:.1f}", value))); }
  void addPercentage(const std::string& label, double value) override { row(localized(label), text(fmt::format("{:.1f}%", value))); }
  void addQuantity(const std::string& label, int value, int capacity) override {
    row(localized(label), text(fmt::format("{} / {}", value, capacity)));
  }
  void addPrefixedQuantity(const std::string& prefix, const std::string& label, int value, int capacity) override {
    row(text(prefix) + " " + localized(label), text(fmt::format("{} / {}", value, capacity)));
  }
};

class CityWorld final : public godot::RefCounted {
  GDCLASS(CityWorld, godot::RefCounted)

  std::unique_ptr<DesktopWorld> city;
  DesktopWorld& world() const {
    if(!city) throw std::runtime_error("No city loaded.");
    return *city;
  }
  const MapTile* tile(Vector2i v) const {
    return city && city->map.is_inside(point(v)) ? city->map(point(v)) : nullptr;
  }

protected:
  static void _bind_methods() {
    ClassDB::bind_method(D_METHOD("new_city", "climate", "side", "village", "trees"), &CityWorld::new_city,
                         DEFVAL(0), DEFVAL(100), DEFVAL(true), DEFVAL(true));
    ClassDB::bind_method(D_METHOD("load_city", "path"), &CityWorld::load_city);
    ClassDB::bind_method(D_METHOD("save_city", "path"), &CityWorld::save_city);
    ClassDB::bind_method(D_METHOD("step"), &CityWorld::step);
    ClassDB::bind_method(D_METHOD("set_vehicle_traffic", "mode"), &CityWorld::set_vehicle_traffic);
    ClassDB::bind_method(D_METHOD("advance_animation", "millis"), &CityWorld::advance_animation);
    ClassDB::bind_method(D_METHOD("advance_traffic", "millis"), &CityWorld::advance_animation);
    ClassDB::bind_method(D_METHOD("snapshot"), &CityWorld::snapshot);
    ClassDB::bind_method(D_METHOD("tools"), &CityWorld::tools);
    ClassDB::bind_method(D_METHOD("preview", "id", "position"), &CityWorld::preview);
    ClassDB::bind_method(D_METHOD("apply", "id", "position"), &CityWorld::apply);
    ClassDB::bind_method(D_METHOD("preview_batch", "id", "positions"), &CityWorld::preview_batch);
    ClassDB::bind_method(D_METHOD("apply_batch", "id", "positions"), &CityWorld::apply_batch);
    ClassDB::bind_method(D_METHOD("inspect_tile", "position", "production"), &CityWorld::inspect_tile, DEFVAL(false));
    ClassDB::bind_method(D_METHOD("set_focus", "position"), &CityWorld::set_focus);
    ClassDB::bind_method(D_METHOD("survey_coal"), &CityWorld::survey_coal);
    ClassDB::bind_method(D_METHOD("launch_rocket", "position"), &CityWorld::launch_rocket);
    ClassDB::bind_method(D_METHOD("trade_rules", "position"), &CityWorld::trade_rules);
    ClassDB::bind_method(D_METHOD("set_trade", "position", "id", "buy", "sell"), &CityWorld::set_trade);
    ClassDB::bind_method(D_METHOD("overlay", "id", "commodity"), &CityWorld::overlay, DEFVAL(0));
    ClassDB::bind_method(D_METHOD("finances"), &CityWorld::finances);
    ClassDB::bind_method(D_METHOD("history"), &CityWorld::history);
    ClassDB::bind_method(D_METHOD("take_events"), &CityWorld::take_events);
    ClassDB::bind_method(D_METHOD("tile_resource", "position"), &CityWorld::tile_resource);
  }

public:
  String new_city(int climate, int side, bool village, bool trees) {
    return run([&] {
      if(climate < 0 || climate > 3) throw std::invalid_argument("Unknown climate.");
      city_settings settings{village, !trees};
      city = generateDesktopCity(static_cast<DesktopClimate>(climate), &settings, side);
    });
  }
  String load_city(String path) {
    return run([&] {
      try {
        city = DesktopWorld::load(file_path(path));
      } catch(const std::exception& error) {
        ERR_PRINT(text(error.what()));
        throw std::runtime_error("The city could not be loaded.");
      }
    });
  }
  String save_city(String path) const {
    if(!city) return localized("No city loaded.");
    return run([&] {
      try {
        world().save(file_path(path));
      } catch(const std::exception& error) {
        ERR_PRINT(text(error.what()));
        throw std::runtime_error("The city could not be saved.");
      }
    });
  }
  String step() { return run([&] { world().do_time_step(); }); }

  void set_vehicle_traffic(int mode) {
    if(!city) return;
    using Traffic = World::VehicleTraffic;
    city->vehicleTraffic = mode == 2 ? Traffic::Normal : mode == 1 ? Traffic::Reduced : Traffic::Disabled;
  }

  godot::PackedVector3Array advance_animation(int64_t millis) {
    godot::PackedVector3Array poses;
    if(!city) return poses;
    try {
      auto cars = rustcity::advance_animation(city->engine(), millis);
      poses.resize(cars.size());
      for(std::size_t i = 0; i < cars.size(); ++i)
        poses.set(i, godot::Vector3(cars[i].x, cars[i].heading, cars[i].z));
    } catch(const std::exception& error) {
      ERR_PRINT(text(error.what()));
    }
    return poses;
  }

  Dictionary snapshot() const {
    Dictionary result;
    if(!city) return result;
    const auto& map = city->map;
    const auto side = map.len();
    PackedInt32Array tiles, anchors, pollution;
    tiles.resize(int64_t(side) * side);
    anchors.resize(tiles.size());
    pollution.resize(tiles.size());
    Dictionary rockets;
    godot::PackedByteArray operating;
    operating.resize(tiles.size());
    auto* production = operating.ptrw();
    auto* groups = tiles.ptrw();
    auto* roots = anchors.ptrw();
    auto* emissions = pollution.ptrw();
    int64_t population = city->people_pool;
    std::array<Stats::Inventory<int64_t>, STUFF_COUNT> inventory{};
    for(const auto& t : map) {
      const auto* building = t.getReportingConstruction();
      const auto anchor = building ? building->point : t.point;
      if(const auto* root = t.getConstruction()) {
        if(t.is_residence()) population += static_cast<const Residence*>(root)->local_population;
        for(auto commodity = STUFF_INIT; commodity < STUFF_COUNT; ++commodity) {
          auto& stock = inventory[commodity];
          stock.amount += root->commodityCount[commodity];
          stock.capacity += root->getConstructionGroup()->commodityRuleCount[commodity].maxload;
        }
      }
      const auto index = t.point.y * side + t.point.x;
      groups[index] = t.getGroup();
      roots[index] = anchor.y * side + anchor.x;
      emissions[index] = t.pollution;
      production[index] = 0;
      if(t.getConstruction() && (groups[index] == GROUP_COAL_POWER || groups[index] == GROUP_INDUSTRY_L || groups[index] == GROUP_INDUSTRY_H))
        production[index] = rustcity::factory_operating(city->engine(), {anchor.x, anchor.y});
      if(t.getConstruction() && groups[index] == GROUP_ROCKET)
        rockets[index] = rustcity::rocket_frame(city->engine(), {anchor.x, anchor.y});
    }
    result["rocket_frames"] = rockets;
    result["tiles"] = tiles;
    result["operating"] = operating;
    result["anchors"] = anchors;
    result["pollution"] = pollution;
    result["side"] = side;
    result["time"] = city->total_time;
    result["money"] = city->total_money;
    const auto& pop = city->stats.population;
    result["population"] = population;
    result["homeless"] = city->people_pool;
    result["tech"] = city->tech_level;
    result["tech_percent"] = city->tech_level * 100.0 / MAX_TECH_LEVEL;
    result["highest_tech_percent"] = city->stats.highest_tech_level * 100.0 / MAX_TECH_LEVEL;
    result["scenario"] = text(city->given_scene);
    result["unemployment"] = pop.unemployed_m / NUMOF_DAYS_IN_MONTH;
    result["population_record"] = pop.max_pop_ever;
    result["evacuated"] = pop.evacuated_t;
    result["births"] = pop.births_t;
    result["deaths"] = pop.starve_deaths_t + pop.pollution_deaths_t;
    result["starvation_deaths"] = pop.starve_deaths_t;
    result["pollution_deaths"] = pop.pollution_deaths_t;
    result["unemployment_years"] = static_cast<double>(pop.unemployed_days_t) / NUMOF_DAYS_IN_YEAR;
    result["rockets"] = city->rockets_launched;
    result["successful_rockets"] = city->rockets_launched_success;
    PackedInt32Array counts;
    counts.resize(city->stats.groupCount.size());
    std::copy(city->stats.groupCount.begin(), city->stats.groupCount.end(), counts.ptrw());
    result["building_counts"] = counts;
    Array stocks;
    for(const auto& stock : inventory) {
      Dictionary entry;
      entry["amount"] = stock.amount;
      entry["capacity"] = stock.capacity;
      stocks.append(entry);
    }
    result["inventory"] = stocks;
    result["sustainable"] = city->stats.sustainability.sustainable;
    result["coal_survey_done"] = bool(city->coal_survey_done);
    result["focus"] = Vector2i(map.recentPoint.x, map.recentPoint.y);
    return result;
  }

  godot::PackedColorArray overlay(int id, int commodity) const {
    godot::PackedColorArray result;
    if(!city || id < MapOverlay::NORMAL || id >= MapOverlay::MAX
        || commodity < STUFF_INIT || commodity >= STUFF_COUNT) return result;
    const int side = city->map.len();
    result.resize(int64_t(side) * side);
    auto* colours = result.ptrw();
    for(const auto& t : city->map) {
      const auto c = MapOverlay::colour(*city, t, static_cast<MapOverlay::DisplayMode>(id),
                                        static_cast<Commodity>(commodity));
      colours[t.point.y * side + t.point.x] = godot::Color::from_rgba8(c.r, c.g, c.b);
    }
    return result;
  }
  Dictionary finances() const {
    Dictionary result, income, expenses;
    if(!city) return result;
    const auto& i = city->stats.income;
    income["Income tax"] = i.income_tax.stat;
    income["Coal tax"] = i.coal_tax.stat;
    income["Goods tax"] = i.goods_tax.stat;
    income["Exports"] = i.export_tax.stat;
    const auto& e = city->stats.expenses;
    expenses["Construction"] = e.construction.stat;
    expenses["Coal survey"] = e.coalSurvey.stat;
    expenses["Imports"] = e.import.stat;
    expenses["Unemployment"] = e.unemployment.stat;
    expenses["Transport"] = e.transport.stat;
    expenses["Windmills"] = e.windmill.stat;
    expenses["Universities"] = e.university.stat;
    expenses["Recycling"] = e.recycle.stat;
    expenses["Deaths"] = e.deaths.stat;
    expenses["Health"] = e.health.stat;
    expenses["Rockets"] = e.rockets.stat;
    expenses["Schools"] = e.school.stat;
    expenses["Fire stations"] = e.firestation.stat;
    expenses["Sports"] = e.cricket.stat;
    expenses["Interest"] = e.interest.stat;
    result["income"] = income;
    result["expenses"] = expenses;
    return result;
  }
  Dictionary history() const {
    if(!city) return {};
    const auto& history = city->stats.history;
    const auto completed_months = std::max(0, city->total_time / NUMOF_DAYS_IN_MONTH);
    const auto missing = std::numeric_limits<double>::quiet_NaN();
    auto export_series = [&](const char* key, const auto& source, const auto convert) {
      godot::PackedFloat64Array values;
      const auto count = std::min(source.size(), std::size_t(completed_months));
      const auto present = rustcity::view::history_presence(city->engine(), key, count);
      values.resize(count);
      auto* output = values.ptrw();
      bool has_values = false;
      for(std::size_t i = 0; i < count; ++i) {
        output[i] = present[i] ? convert(source[i]) : missing;
        has_values |= std::isfinite(output[i]);
      }
      if(!has_values) values.clear();
      return values;
    };
    const auto raw = [](int value) { return double(value); };
    const auto people = [](int days) { return double(days) / NUMOF_DAYS_IN_MONTH; };
    const auto technology = [](int value) { return value * 100.0 / MAX_TECH_LEVEL; };
    const auto fill = [missing](const Stats::Inventory<>& stock) {
      return stock.capacity > 0 ? stock.amount * 100.0 / stock.capacity : missing;
    };
    Dictionary series;
    series["population"] = export_series("pop", history.pop, people);
    series["homeless"] = export_series("ppool", history.ppool, people);
    series["starving"] = export_series("starve", history.starve, people);
    series["unemployed"] = export_series("nojobs", history.nojobs, people);
    series["births"] = export_series("births", history.births, raw);
    series["deaths"] = export_series("deaths", history.deaths, raw);
    series["unnatural_deaths"] = export_series("unnat_deaths", history.unnat_deaths, raw);
    series["balance"] = export_series("money", history.money, raw);
    series["tech"] = export_series("tech", history.tech, technology);
    series["pollution"] = export_series("pollution", history.pollution, raw);
    series["occupancy"] = export_series("tenants", history.tenants, fill);
    Array commodities;
    for(auto id = STUFF_INIT; id < STUFF_COUNT; ++id) {
      Dictionary commodity;
      commodity["name"] = localized(commodityName(id));
      commodity["values"] = export_series(commodityStandardName(id), history.inventory[id], fill);
      commodities.append(commodity);
    }
    const auto& sustain = city->stats.sustainability;
    Array criteria;
    const auto criterion = [&](const char* name, int years, int goal) {
      Dictionary progress;
      progress["name"] = localized(name);
      progress["years"] = years;
      progress["goal"] = goal;
      criteria.append(progress);
    };
    criterion("Coal and ore mining", sustain.mining_years, SUST_ORE_COAL_YEARS_NEEDED);
    criterion("Port trade", sustain.trade_years, SUST_PORT_YEARS_NEEDED);
    criterion("City finances", sustain.money_years, SUST_MONEY_YEARS_NEEDED);
    criterion("Population", sustain.population_years, SUST_POP_YEARS_NEEDED);
    criterion("Technology", sustain.tech_years, SUST_TECH_YEARS_NEEDED);
    criterion("Fire protection", sustain.fire_years, SUST_FIRE_YEARS_NEEDED);
    Dictionary result;
    result["last_month"] = completed_months - 1;
    result["series"] = series;
    result["commodities"] = commodities;
    result["sustainability"] = criteria;
    result["sustainable"] = sustain.sustainable;
    return result;
  }
  Array tools() const {
    initializeModules();
    Array list;
    auto append = [&](int id, String name, int size, int base_cost = 0, int required_tech = 0) {
      Dictionary item;
      item["id"] = id;
      item["name"] = name;
      item["size"] = size;
      item["base_cost"] = base_cost;
      item["required_tech"] = required_tech;
      list.append(item);
    };
    append(inspect, localized("Inspect"), 1);
    append(demolish, localized("Bulldoze"), 1);
    append(evacuate, localized("Evacuate"), 1);
    append(water, localized("Water"), 1, GROUP_WATER_COST);
    for(int id = 0; id < NUM_OF_GROUPS; ++id) {
      auto op = operation(id);
      if(auto* g = op.constructionGroup) append(id, localized(g->name), g->size, g->cost, g->tech);
    }
    return list;
  }
  Dictionary preview(int id, Vector2i v) const {
    Dictionary result;
    auto op = operation(id);
    result["allowed"] = false;
    result["cost"] = 0;
    result["size"] = op.constructionGroup ? op.constructionGroup->size : 1;
    if(!tile(v) || op.action == UserOperation::ACTION_UNKNOWN) return result;
    Message::ptr why;
    result["allowed"] = desktopPreview(*city, op, point(v), why);
    result["cost"] = desktopCost(*city, op, point(v));
    if(why) result["reason"] = message_text(why);
    return result;
  }
  String apply(int id, Vector2i v) {
    return run([&] {
      auto& w = world();
      auto op = operation(id);
      Message::ptr why;
      if(op.action == UserOperation::ACTION_UNKNOWN) throw std::invalid_argument("Unknown tool.");
      if(!desktopPreview(w, op, point(v), why)) {
        if(why) why->throwEx();
        throw std::runtime_error("This action is not possible here.");
      }
      desktopApply(w, op, point(v));
    }, city.get());
  }
  Array preview_batch(int id, const TypedArray<Vector2i>& positions) const {
    return action_batch(id, positions, false);
  }
  Array apply_batch(int id, const TypedArray<Vector2i>& positions) {
    return action_batch(id, positions, true);
  }
private:
  Array action_batch(int id, const TypedArray<Vector2i>& positions, bool apply) const {
    Array results;
    auto op = operation(id);
    if(!city || op.action == UserOperation::ACTION_UNKNOWN) {
      for(int64_t i = 0; i < positions.size(); ++i) {
        Dictionary entry;
        entry["allowed"] = false;
        entry["cost"] = 0;
        entry["size"] = 1;
        entry["group"] = 0;
        entry["anchor"] = positions[i];
        entry["reason"] = localized(city ? "Unknown tool." : "No city loaded.");
        results.append(entry);
      }
      return results;
    }
    std::vector<rustcity::Point> targets;
    targets.reserve(positions.size());
    for(int64_t i = 0; i < positions.size(); ++i) {
      const Vector2i v = positions[i];
      targets.push_back({v.x, v.y});
    }
    const rust::Slice<const rustcity::Point> points(targets.data(), targets.size());
    const auto batch = apply ? desktopApplyBatch(*city, op, points) : desktopPreviewBatch(*city, op, points);
    using F = rustcity::Failure;
    for(const auto& result : batch) {
      Dictionary entry;
      entry["allowed"] = result.action.error == F::None;
      entry["cost"] = result.action.cost;
      entry["size"] = result.size;
      entry["group"] = result.action.group;
      entry["anchor"] = Vector2i(result.anchor.x, result.anchor.y);
      if(result.action.error == F::DecisionTrace)
        entry["reason"] = localized("This action is not possible here.");
      else if(auto why = desktopActionMessage(result.action))
        entry["reason"] = message_text(why);
      if(result.action.error == F::NoCredit) entry["warning"] = "out_of_credit";
      if(result.action.error == F::OutOfMoney) entry["warning"] = "out_of_money";
      results.append(entry);
    }
    return results;
  }
public:
  String inspect_tile(Vector2i v, bool production) const {
    const auto* t = tile(v);
    if(!t) return {};
    Report report;
    if(const auto* building = t->getReportingConstruction()) {
      renderRustReport(city->engine(), building->point, production, report);
    } else {
      report.addLabel(t->getTileConstructionGroup()->name);
      report.addInteger("Elevation", t->ground.altitude);
      report.addInteger("Ore", t->ore_reserve);
      if(city->coal_survey_done) report.addInteger("Coal", t->coal_reserve);
    }
    report.addInteger("Air pollution", t->pollution);
    return report.out.strip_edges();
  }
  void set_focus(Vector2i v) {
    if(tile(v)) desktopFocus(*city, point(v));
  }
  String survey_coal() { return run([&] { world().do_coal_survey(); }, city.get()); }
  String launch_rocket(Vector2i v) {
    return run([&] {
      const auto* t = tile(v);
      const auto* building = t ? t->getReportingConstruction() : nullptr;
      if(!building || !world().launchRocket(building->point))
        throw std::runtime_error("No rocket is ready to launch here.");
    });
  }
  Array trade_rules(Vector2i v) const {
    Array result;
    const auto* t = tile(v);
    if(!t) return result;
    const auto* building = t->getReportingConstruction();
    const auto* market = dynamic_cast<const Market*>(building);
    const auto* port = dynamic_cast<const Port*>(building);
    if(!market && !port) return result;
    const auto& rules = market ? market->commodityRuleCount : port->commodityRuleCount;
    for(int i = STUFF_INIT; i < STUFF_COUNT; ++i) {
      if(!rules[i].maxload) continue;
      Dictionary item;
      item["id"] = i;
      item["name"] = localized(commodityName(static_cast<Commodity>(i)));
      item["buy"] = rules[i].take;
      item["sell"] = rules[i].give;
      result.append(item);
    }
    return result;
  }
  String set_trade(Vector2i v, int id, bool buy, bool sell) {
    return run([&] {
      auto& w = world();
      if(id < STUFF_INIT || id >= STUFF_COUNT) throw std::invalid_argument("Unknown commodity.");
      const auto commodity = static_cast<Commodity>(id);
      if(!w.setMarketTrade(point(v), commodity, buy, sell) && !w.setPortTrade(point(v), commodity, buy, sell))
        throw std::runtime_error("This building cannot trade this commodity.");
    });
  }
  String tile_resource(Vector2i v) const {
    const auto* t = tile(v);
    if(!t) return {};
    const auto* building = t->getReportingConstruction();
    const auto* resource = building ? building->getFrame().resourceGroup : t->getTileResourceGroup();
    return resource ? text(resource->resourceID) : String{};
  }

  Array take_events() {
    Array result;
    if(!city) return result;
    while(auto message = city->popMessage()) {
      Dictionary event;
      event["text"] = message_text(message);
      if(dynamic_message_cast<SustainableEconomyMessage>(message))
        event["kind"] = "economy";
      else if(dynamic_message_cast<NoPeopleLeftMessage>(message))
        event["kind"] = "loss";
      else if(dynamic_message_cast<RocketReadyMessage>(message))
        event["kind"] = "rocket_ready";
      else if(auto fire = dynamic_message_cast<FireStartedMessage>(message)) {
        const auto cell = fire->getPoint();
        event["kind"] = "fire";
        event["cell"] = Vector2i(cell.x, cell.y);
        event["group"] = fire->getGroup().group;
      } else if(auto money = dynamic_message_cast<OutOfMoneyMessage>(message))
        event["kind"] = money->isOutOfCredit() ? "out_of_credit" : "out_of_money";
      else if(auto rocket = dynamic_message_cast<RocketResultMessage>(message)) {
        using Result = RocketResultMessage::LaunchResult;
        switch(rocket->getResult()) {
          case Result::FAIL: event["kind"] = "rocket_crash"; break;
          case Result::SUCCESS: event["kind"] = "rocket_success"; break;
          case Result::EVAC: event["kind"] = "rocket_evacuation"; break;
          case Result::EVAC_WIN: event["kind"] = "evacuation"; break;
        }
      }
      result.append(event);
    }
    return result;
  }
};

void initialize(godot::ModuleInitializationLevel level) {
  if(level == godot::MODULE_INITIALIZATION_LEVEL_SCENE)
    ClassDB::register_class<CityWorld>();
}
void terminate(godot::ModuleInitializationLevel) {}
}

extern "C" GDExtensionBool GDE_EXPORT lincity_library_init(
    GDExtensionInterfaceGetProcAddress get_proc_address, GDExtensionClassLibraryPtr library,
    GDExtensionInitialization* initialization) {
  godot::GDExtensionBinding::InitObject init(get_proc_address, library, initialization);
  init.register_initializer(initialize);
  init.register_terminator(terminate);
  init.set_minimum_library_initialization_level(godot::MODULE_INITIALIZATION_LEVEL_SCENE);
  return init.init();
}
