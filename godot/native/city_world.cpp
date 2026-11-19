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

#include <godot_cpp/variant/packed_color_array.hpp>
#include "rustcity-godot/src/view.rs.h"

#include <fmt/format.h>

#include <algorithm>
#include <array>
#include <cmath>
#include <limits>
#include <optional>
#include <stdexcept>
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
String text_rust(const rust::String& value) { return String::utf8(value.data(), value.size()); }
String localized(const std::string& value) {
  return godot::TranslationServer::get_singleton()->translate(text(value));
}

String localized_rust(const rust::String& value) {
  return godot::TranslationServer::get_singleton()->translate(text_rust(value));
}

template<class... Args>
String localized_format(const std::string& key, const Args&... args) {
  const auto format = localized(key).utf8();
  return text(fmt::format(fmt::runtime(format.get_data()), args...));
}

const auto calendar = rustcity::view::calendar();

using rustcity::ActionKind;
using rustcity::Failure;

String action_message(const rustcity::ActionResult& action) {
  switch(action.error) {
    case Failure::None: return {};
    case Failure::OutsideMap: return localized("This space is outside the map.");
    case Failure::SpaceOccupied: return localized("This space is occupied.");
    case Failure::NothingToEvacuate: return localized("There is nothing to evacuate here.");
    case Failure::CannotEvacuate:
    case Failure::ProtectedFire: {
      const auto name = localized_rust(rustcity::view::group_name(action.group)).utf8();
      return localized_format(action.error == Failure::CannotEvacuate
        ? "Cannot evacuate {}." : "Cannot bulldoze this {}.", name.get_data());
    }
    case Failure::NoCredit: return localized("Out of Credit");
    case Failure::OutOfMoney: return localized("Out of Money");
    case Failure::NotEnoughTech:
      return localized_format("Technology required: {:.1f}%. Current level: {:.1f}%.",
        action.required_tech * 100.0 / calendar.max_tech,
        action.current_tech * 100.0 / calendar.max_tech);
    case Failure::NotEnoughStudents: return localized("There are not enough students. Build more schools first.");
    case Failure::DesertHere: return localized("This space is desert and has no water.");
    case Failure::RequiresRiver: return localized("A port requires a river along the east side.");
    case Failure::NoOre: return localized("There are no ore reserves left here.");
    case Failure::NonemptyTip: return localized("You cannot bulldoze this land fill because it is full of waste.");
    case Failure::IncompleteMonument: return localized("You cannot bulldoze this monument because it is still under construction.");
    case Failure::DecisionTrace: return localized("This action is not possible here.");
    default: return localized("Unknown tool.");
  }
}

rustcity::Point point(Vector2i v) { return {v.x, v.y}; }
std::string file_path(const String& path) {
  auto absolute = godot::ProjectSettings::get_singleton()->globalize_path(path).utf8();
  return absolute.get_data();
}

template<class Command> String run(Command command) {
  try {
    command();
    return {};
  } catch(const std::exception& error) {
    return localized(error.what());
  }
}

enum SpecialTool { water = -4, evacuate, demolish, inspect };

ActionKind operation(int id) {
  switch(id) {
    case inspect: return ActionKind::Query;
    case demolish: return ActionKind::Bulldoze;
    case evacuate: return ActionKind::Evacuate;
    case water: return ActionKind::Flood;
    default: return ActionKind::Build;
  }
}

constexpr std::array commodity_names{
  "Food", "Labor", "Coal", "Goods", "Ore", "Steel", "Waste", "Lo-Volt", "Hi-Volt", "Water"
};
constexpr std::array commodity_keys{
  "food", "labor", "coal", "goods", "ore", "steel", "waste", "lovolt", "hivolt", "water"
};

void report_row(String& output, const String& label, const String& value) {
  output += label + String("  ") + value + String("\n");
}

void append_report(String& output, const rustcity::view::Row& row) {
  using K = rustcity::view::RowKind;
  const auto label = localized_rust(row.label);
  switch(row.kind) {
    case K::Blank: output += "\n"; break;
    case K::Label: output += label + String("\n"); break;
    case K::Text: report_row(output, label, localized_rust(row.text)); break;
    case K::Integer: report_row(output, label, String::num_int64(row.value)); break;
    case K::Decimal: report_row(output, label, text(fmt::format("{:.1f}", row.number))); break;
    case K::Percentage: report_row(output, label, text(fmt::format("{:.1f}%", row.number))); break;
    case K::Quantity:
      report_row(output, label, text(fmt::format("{} / {}", row.value, row.capacity)));
      break;
    case K::Prefixed:
      report_row(output, text_rust(row.text) + " " + label, text(fmt::format("{} / {}", row.value, row.capacity)));
      break;
  }
}

template<class Packed, class T>
Packed packed_array(const rust::Vec<T>& values) {
  Packed output;
  output.resize(values.size());
  std::copy(values.begin(), values.end(), output.ptrw());
  return output;
}

class CityWorld final : public godot::RefCounted {
  GDCLASS(CityWorld, godot::RefCounted)

  std::optional<rust::Box<rustcity::Engine>> city;
  Array pending_events;
  rustcity::Traffic traffic = rustcity::Traffic::Disabled;

  rustcity::Engine& world() {
    if(!city) throw std::runtime_error("No city loaded.");
    return **city;
  }
  const rustcity::Engine& world() const {
    if(!city) throw std::runtime_error("No city loaded.");
    return **city;
  }
  bool has_tile(Vector2i v) const {
    if(!city) return false;
    const auto side = rustcity::side(world());
    return v.x >= 0 && v.y >= 0 && v.x < side && v.y < side;
  }

  void remember_warning(Failure failure) {
    if(failure != Failure::NoCredit && failure != Failure::OutOfMoney) return;
    collect_events();
    Dictionary event;
    event["kind"] = failure == Failure::NoCredit ? "out_of_credit" : "out_of_money";
    event["text"] = localized(failure == Failure::NoCredit ? "Out of Credit" : "Out of Money");
    pending_events.append(event);
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
      city = rustcity::generate(side, static_cast<rustcity::Climate>(climate), village, trees);
      pending_events.clear();
      traffic = rustcity::Traffic::Disabled;
    });
  }
  String load_city(String path) {
    return run([&] {
      try {
        city = rustcity::load(file_path(path));
        pending_events.clear();
        traffic = rustcity::Traffic::Disabled;
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
        rustcity::save(world(), file_path(path));
      } catch(const std::exception& error) {
        ERR_PRINT(text(error.what()));
        throw std::runtime_error("The city could not be saved.");
      }
    });
  }
  String step() {
    return run([&] {
      rustcity::set_traffic(world(), traffic);
      rustcity::tick(world());
    });
  }

  void set_vehicle_traffic(int mode) {
    if(!city) return;
    using T = rustcity::Traffic;
    traffic = mode == 2 ? T::Normal : mode == 1 ? T::Reduced : T::Disabled;
  }

  godot::PackedVector3Array advance_animation(int64_t millis) {
    godot::PackedVector3Array poses;
    if(!city) return poses;
    try {
      auto cars = rustcity::advance_animation(world(), millis);
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
    const auto view = rustcity::view::snapshot(world());
    const auto stats = rustcity::view::statistics(world());
    Dictionary rockets;
    for(const auto& rocket : view.rockets) rockets[rocket.index] = rocket.frame;
    result["rocket_frames"] = rockets;
    result["tiles"] = packed_array<PackedInt32Array>(view.groups);
    result["operating"] = packed_array<godot::PackedByteArray>(view.operating);
    result["anchors"] = packed_array<PackedInt32Array>(view.anchors);
    result["pollution"] = packed_array<PackedInt32Array>(view.pollution);
    result["side"] = rustcity::side(world());
    result["time"] = stats.day;
    result["money"] = stats.balance;
    result["population"] = view.population;
    result["homeless"] = stats.people_pool;
    result["tech"] = stats.tech;
    result["tech_percent"] = stats.tech * 100.0 / calendar.max_tech;
    result["highest_tech_percent"] = stats.highest_tech_level * 100.0 / calendar.max_tech;
    result["scenario"] = text_rust(stats.name);
    const auto& pop = stats.population;
    result["unemployment"] = pop.unemployed_m / calendar.days_per_month;
    result["population_record"] = pop.max_pop_ever;
    result["evacuated"] = pop.evacuated_t;
    result["births"] = pop.births_t;
    result["deaths"] = pop.starve_deaths_t + pop.pollution_deaths_t;
    result["starvation_deaths"] = pop.starve_deaths_t;
    result["pollution_deaths"] = pop.pollution_deaths_t;
    result["unemployment_years"] = static_cast<double>(pop.unemployed_days_t) / calendar.days_per_year;
    result["rockets"] = stats.launched;
    result["successful_rockets"] = stats.successes;
    result["building_counts"] = packed_array<PackedInt32Array>(stats.group_count);
    Array stocks;
    for(const auto& stock : view.inventory) {
      Dictionary entry;
      entry["amount"] = stock.amount;
      entry["capacity"] = stock.capacity;
      stocks.append(entry);
    }
    result["inventory"] = stocks;
    result["sustainable"] = stats.sustainability.sustainable;
    result["coal_survey_done"] = stats.survey;
    result["focus"] = Vector2i(stats.focus_x, stats.focus_y);
    return result;
  }

  godot::PackedColorArray overlay(int id, int commodity) const {
    godot::PackedColorArray result;
    if(!city) return result;
    const auto rgb = rustcity::overlay(world(), id, commodity);
    result.resize(rgb.size());
    auto* colours = result.ptrw();
    for(std::size_t i = 0; i < rgb.size(); ++i)
      colours[i] = godot::Color::from_rgba8(rgb[i] >> 16, (rgb[i] >> 8) & 255, rgb[i] & 255);
    return result;
  }
  Dictionary finances() const {
    Dictionary result, income, expenses;
    if(!city) return result;
    const auto stats = rustcity::view::statistics(world());
    const auto& i = stats.income;
    income["Income tax"] = i.income_tax;
    income["Coal tax"] = i.coal_tax;
    income["Goods tax"] = i.goods_tax;
    income["Exports"] = i.export_tax;
    const auto& e = stats.expenses;
    expenses["Construction"] = e.construction;
    expenses["Coal survey"] = e.coal_survey;
    expenses["Imports"] = e.import;
    expenses["Unemployment"] = e.unemployment;
    expenses["Transport"] = e.transport;
    expenses["Windmills"] = e.windmill;
    expenses["Universities"] = e.university;
    expenses["Recycling"] = e.recycle;
    expenses["Deaths"] = e.deaths;
    expenses["Health"] = e.health;
    expenses["Rockets"] = e.rockets;
    expenses["Schools"] = e.school;
    expenses["Fire stations"] = e.firestation;
    expenses["Sports"] = e.cricket;
    expenses["Interest"] = e.interest;
    result["income"] = income;
    result["expenses"] = expenses;
    return result;
  }
  Dictionary history() const {
    if(!city) return {};
    const auto history = rustcity::view::history(world());
    const auto stats = rustcity::view::statistics(world());
    const auto completed_months = std::max(0, stats.day / calendar.days_per_month);
    const auto missing = std::numeric_limits<double>::quiet_NaN();
    auto export_series = [&](const char* key, const auto& source, const auto convert) {
      godot::PackedFloat64Array values;
      const auto count = std::min(source.size(), std::size_t(completed_months));
      const auto present = rustcity::view::history_presence(world(), key, count);
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
    const auto people = [](int days) { return double(days) / calendar.days_per_month; };
    const auto technology = [](int value) { return value * 100.0 / calendar.max_tech; };
    const auto fill = [missing](const rustcity::view::Inventory& stock) {
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
    for(std::size_t id = 0; id < commodity_names.size(); ++id) {
      Dictionary commodity;
      commodity["name"] = localized(commodity_names[id]);
      commodity["values"] = export_series(commodity_keys[id], history.inventory[id].samples, fill);
      commodities.append(commodity);
    }
    const auto& sustain = stats.sustainability;
    Array criteria;
    const auto criterion = [&](const char* name, int years, int goal) {
      Dictionary progress;
      progress["name"] = localized(name);
      progress["years"] = years;
      progress["goal"] = goal;
      criteria.append(progress);
    };
    criterion("Coal and ore mining", sustain.mining_years, sustain.goals[0]);
    criterion("Port trade", sustain.trade_years, sustain.goals[1]);
    criterion("City finances", sustain.money_years, sustain.goals[2]);
    criterion("Population", sustain.population_years, sustain.goals[3]);
    criterion("Technology", sustain.tech_years, sustain.goals[4]);
    criterion("Fire protection", sustain.fire_years, sustain.goals[5]);
    Dictionary result;
    result["last_month"] = completed_months - 1;
    result["series"] = series;
    result["commodities"] = commodities;
    result["sustainability"] = criteria;
    result["sustainable"] = sustain.sustainable;
    return result;
  }
  Array tools() const {
    Array list;
    for(const auto& tool : rustcity::view::tools()) {
      Dictionary item;
      item["id"] = tool.id;
      item["name"] = localized_rust(tool.name);
      item["size"] = tool.size;
      item["base_cost"] = tool.base_cost;
      item["required_tech"] = tool.required_tech;
      list.append(item);
    }
    return list;
  }
  Dictionary preview(int id, Vector2i v) const {
    const auto tool = rustcity::view::tool(id);
    Dictionary result;
    result["allowed"] = false;
    result["cost"] = 0;
    result["size"] = std::max<int>(1, tool.size);
    if(!has_tile(v) || !tool.size) return result;
    const auto action = rustcity::preview(world(), operation(id), std::max(0, id), point(v));
    result["allowed"] = action.error == Failure::None;
    result["cost"] = action.cost;
    if(action.error != Failure::None) result["reason"] = action_message(action);
    return result;
  }
  String apply(int id, Vector2i v) {
    if(!city) return localized("No city loaded.");
    if(!rustcity::view::tool(id).size) return localized("Unknown tool.");
    const auto kind = operation(id);
    const auto group = std::max(0, id);
    auto action = rustcity::preview(world(), kind, group, point(v));
    if(action.error == Failure::None) action = rustcity::apply(world(), kind, group, point(v));
    remember_warning(action.error);
    return action_message(action);
  }
  Array preview_batch(int id, const TypedArray<Vector2i>& positions) {
    return action_batch(id, positions, false);
  }
  Array apply_batch(int id, const TypedArray<Vector2i>& positions) {
    return action_batch(id, positions, true);
  }
private:
  Array action_batch(int id, const TypedArray<Vector2i>& positions, bool apply) {
    Array results;
    if(!city || !rustcity::view::tool(id).size) {
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
    const auto kind = operation(id);
    const auto group = std::max(0, id);
    const auto batch = apply ? rustcity::apply_batch(world(), kind, group, points)
                             : rustcity::preview_batch(world(), kind, group, points);
    using F = rustcity::Failure;
    for(const auto& result : batch) {
      Dictionary entry;
      entry["allowed"] = result.action.error == F::None;
      entry["cost"] = result.action.cost;
      entry["size"] = result.size;
      entry["group"] = result.action.group;
      entry["anchor"] = Vector2i(result.anchor.x, result.anchor.y);
      if(result.action.error != F::None) entry["reason"] = action_message(result.action);
      if(result.action.error == F::NoCredit) entry["warning"] = "out_of_credit";
      if(result.action.error == F::OutOfMoney) entry["warning"] = "out_of_money";
      results.append(entry);
    }
    return results;
  }
public:
  String inspect_tile(Vector2i v, bool production) const {
    if(!has_tile(v)) return {};
    const auto t = rustcity::view::tile(world(), point(v));
    String output;
    if(t.group != t.terrain) {
      for(const auto& row : rustcity::view::report(world(), t.anchor_x, t.anchor_y, production))
        append_report(output, row);
    } else {
      output += localized_rust(t.name) + "\n";
      report_row(output, localized("Elevation"), String::num_int64(t.altitude));
      report_row(output, localized("Ore"), String::num_int64(t.ore));
      if(rustcity::view::statistics(world()).survey)
        report_row(output, localized("Coal"), String::num_int64(t.coal));
    }
    report_row(output, localized("Air pollution"), String::num_int64(t.pollution));
    return output.strip_edges();
  }
  void set_focus(Vector2i v) {
    if(has_tile(v)) rustcity::set_focus(world(), point(v));
  }
  String survey_coal() {
    if(!city) return localized("No city loaded.");
    const auto action = rustcity::survey_coal(world());
    remember_warning(action.error);
    return action_message(action);
  }
  String launch_rocket(Vector2i v) {
    if(has_tile(v)) {
      const auto t = rustcity::view::tile(world(), point(v));
      if(rustcity::launch(world(), {t.anchor_x, t.anchor_y})) return {};
    }
    return localized("No rocket is ready to launch here.");
  }
  Array trade_rules(Vector2i v) const {
    Array result;
    if(!has_tile(v)) return result;
    for(const auto& rule : rustcity::view::trade_rules(world(), point(v))) {
      Dictionary item;
      item["id"] = rule.id;
      item["name"] = localized(commodity_names[rule.id]);
      item["buy"] = rule.buy;
      item["sell"] = rule.sell;
      result.append(item);
    }
    return result;
  }
  String set_trade(Vector2i v, int id, bool buy, bool sell) {
    if(!city) return localized("No city loaded.");
    if(id < 0 || id >= int(commodity_names.size())) return localized("Unknown commodity.");
    if(rustcity::market_trade(world(), point(v), id, buy, sell)
       || rustcity::port_trade(world(), point(v), id, buy, sell)) return {};
    return localized("This building cannot trade this commodity.");
  }
  String tile_resource(Vector2i v) const {
    return has_tile(v) ? text_rust(rustcity::view::tile(world(), point(v)).resource) : String{};
  }

  Array take_events() {
    collect_events();
    Array result = pending_events;
    pending_events = Array();
    return result;
  }

private:
  void collect_events() {
    if(!city) return;
    using K = rustcity::EventKind;
    for(auto event = rustcity::pop_event(world()); event.kind != K::None;
        event = rustcity::pop_event(world())) {
      Dictionary item;
      const char* kind;
      const char* message;
      switch(event.kind) {
        case K::SustainableEconomy: kind = "economy"; message = "The economy is sustainable."; break;
        case K::NoPeopleLeft: kind = "loss"; message = "The city has lost all its inhabitants."; break;
        case K::RocketReady: kind = "rocket_ready"; message = "A rocket is ready to launch."; break;
        case K::RocketFailed: kind = "rocket_crash"; message = "The rocket crashed."; break;
        case K::RocketSuccess: kind = "rocket_success"; message = "The rocket launched successfully."; break;
        case K::RocketEvacuated: kind = "rocket_evacuation"; message = "1,000 people were evacuated."; break;
        case K::RocketWon: kind = "evacuation"; message = "Everyone has been evacuated from the city."; break;
        case K::OutOfMoney: kind = "out_of_money"; message = "Out of Money"; break;
        case K::FireStarted: {
          item["kind"] = "fire";
          item["cell"] = Vector2i(event.point.x, event.point.y);
          item["group"] = event.group;
          const auto name = localized_rust(rustcity::view::group_name(event.group)).utf8();
          item["text"] = localized_format("A fire has started: {} {}.", name.get_data(),
                                              fmt::format("({},{})", event.point.x, event.point.y));
          pending_events.append(item);
          continue;
        }
        default: continue;
      }
      item["kind"] = kind;
      item["text"] = localized(message);
      pending_events.append(item);
    }
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
