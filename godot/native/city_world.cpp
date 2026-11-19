// Godot owns this façade. Each instance owns an independent simulation.
#include <godot_cpp/classes/project_settings.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/translation_server.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/godot.hpp>
#include <godot_cpp/variant/packed_int32_array.hpp>
#include <godot_cpp/variant/packed_byte_array.hpp>
#include <godot_cpp/variant/packed_vector3_array.hpp>
#include <godot_cpp/variant/packed_string_array.hpp>

#include "DesktopWorld.hpp"
#include "lincity/ConstructionReport.hpp"
#include "MapOverlay.hpp"
#include <godot_cpp/variant/packed_color_array.hpp>
#include "lincity/modules/all_modules.hpp"

#include <fmt/format.h>

namespace {
using godot::Array;
using godot::ClassDB;
using godot::D_METHOD;
using godot::Dictionary;
using godot::PackedInt32Array;
using godot::PackedStringArray;
using godot::String;
using godot::Vector2i;

String text(const std::string& value) { return String::utf8(value.c_str()); }
String localized(const std::string& value) {
  return godot::TranslationServer::get_singleton()->translate(text(value));
}
MapPoint point(Vector2i v) { return {v.x, v.y}; }
std::filesystem::path file_path(const String& path) {
  auto absolute = godot::ProjectSettings::get_singleton()->globalize_path(path);
  return std::filesystem::u8path(absolute.utf8().get_data());
}

template<class Command> String run(Command command) {
  try {
    command();
    return {};
  } catch(const Message::Exception& error) {
    return String::utf8(error.what());
  } catch(const std::exception& error) {
    return String::utf8(error.what());
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
    if(!city) throw std::runtime_error("Keine Stadt geladen.");
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
    ClassDB::bind_method(D_METHOD("advance_animation", "millis"), &CityWorld::advance_animation);
    ClassDB::bind_method(D_METHOD("advance_traffic", "millis"), &CityWorld::advance_animation);
    ClassDB::bind_method(D_METHOD("snapshot"), &CityWorld::snapshot);
    ClassDB::bind_method(D_METHOD("tools"), &CityWorld::tools);
    ClassDB::bind_method(D_METHOD("preview", "id", "position"), &CityWorld::preview);
    ClassDB::bind_method(D_METHOD("apply", "id", "position"), &CityWorld::apply);
    ClassDB::bind_method(D_METHOD("inspect_tile", "position", "production"), &CityWorld::inspect_tile, DEFVAL(false));
    ClassDB::bind_method(D_METHOD("set_focus", "position"), &CityWorld::set_focus);
    ClassDB::bind_method(D_METHOD("survey_coal"), &CityWorld::survey_coal);
    ClassDB::bind_method(D_METHOD("launch_rocket", "position"), &CityWorld::launch_rocket);
    ClassDB::bind_method(D_METHOD("trade_rules", "position"), &CityWorld::trade_rules);
    ClassDB::bind_method(D_METHOD("set_trade", "position", "id", "buy", "sell"), &CityWorld::set_trade);
    ClassDB::bind_method(D_METHOD("overlay", "id", "commodity"), &CityWorld::overlay, DEFVAL(0));
    ClassDB::bind_method(D_METHOD("finances"), &CityWorld::finances);
    ClassDB::bind_method(D_METHOD("take_messages"), &CityWorld::take_messages);
  }

public:
  String new_city(int climate, int side, bool village, bool trees) {
    return run([&] {
      if(climate < 0 || climate > 3) throw std::invalid_argument("Unbekanntes Klima.");
      city_settings settings{village, !trees};
      city = generateDesktopCity(static_cast<DesktopClimate>(climate), &settings, side);
    });
  }
  String load_city(String path) { return run([&] { city = DesktopWorld::load(file_path(path)); }); }
  String save_city(String path) const {
    return run([&] { world().save(file_path(path)); });
  }
  String step() { return run([&] { world().do_time_step(); }); }

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
    for(const auto& t : map) {
      const auto* building = t.getReportingConstruction();
      const auto anchor = building ? building->point : t.point;
      if(t.getConstruction() && t.is_residence())
        population += static_cast<const Residence*>(building)->local_population;
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
    result["unemployment"] = pop.unemployed_m / NUMOF_DAYS_IN_MONTH;
    result["births"] = pop.births_t;
    result["deaths"] = pop.starve_deaths_t + pop.pollution_deaths_t;
    result["rockets"] = city->rockets_launched;
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
  Array tools() const {
    initializeModules();
    Array list;
    auto append = [&](int id, String name, int size) {
      Dictionary item;
      item["id"] = id;
      item["name"] = name;
      item["size"] = size;
      list.append(item);
    };
    append(inspect, "Inspect", 1);
    append(demolish, "Bulldoze", 1);
    append(evacuate, "Evacuate", 1);
    append(water, "Water", 1);
    for(int id = 0; id < NUM_OF_GROUPS; ++id) {
      auto op = operation(id);
      if(auto* g = op.constructionGroup) append(id, localized(g->name), g->size);
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
    if(why) result["reason"] = text(why->str());
    return result;
  }
  String apply(int id, Vector2i v) {
    return run([&] {
      auto& w = world();
      auto op = operation(id);
      Message::ptr why;
      if(op.action == UserOperation::ACTION_UNKNOWN) throw std::invalid_argument("Unbekanntes Werkzeug.");
      if(!desktopPreview(w, op, point(v), why)) {
        if(why) why->throwEx();
        throw std::runtime_error("Diese Aktion ist hier nicht möglich.");
      }
      desktopApply(w, op, point(v));
    });
  }
  String inspect_tile(Vector2i v, bool production) const {
    const auto* t = tile(v);
    if(!t) return {};
    Report report;
    if(const auto* building = t->getReportingConstruction()) {
      renderRustReport(city->engine(), building->point, production, report);
    } else {
      report.addLabel(t->getTileConstructionGroup()->getName());
      report.addInteger("Höhe", t->ground.altitude);
      report.addInteger("Erz", t->ore_reserve);
      if(city->coal_survey_done) report.addInteger("Kohle", t->coal_reserve);
    }
    report.addInteger("Luftverschmutzung", t->pollution);
    return report.out.strip_edges();
  }
  void set_focus(Vector2i v) {
    if(tile(v)) desktopFocus(*city, point(v));
  }
  String survey_coal() { return run([&] { world().do_coal_survey(); }); }
  String launch_rocket(Vector2i v) {
    return run([&] {
      const auto* t = tile(v);
      const auto* building = t ? t->getReportingConstruction() : nullptr;
      if(!building || !world().launchRocket(building->point))
        throw std::runtime_error("Hier ist keine Rakete startbereit.");
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
      if(id < STUFF_INIT || id >= STUFF_COUNT) throw std::invalid_argument("Unbekannte Ware.");
      const auto commodity = static_cast<Commodity>(id);
      if(!w.setMarketTrade(point(v), commodity, buy, sell) && !w.setPortTrade(point(v), commodity, buy, sell))
        throw std::runtime_error("Für dieses Gebäude oder diese Ware ist kein Handel möglich.");
    });
  }
  PackedStringArray take_messages() {
    PackedStringArray result;
    if(city) while(auto message = city->popMessage()) result.append(text(message->str()));
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
