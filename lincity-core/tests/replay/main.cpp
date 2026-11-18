#include "events.hpp"
#include "lincity/UserOperation.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <charconv>
#include <cstdlib>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {
struct Command {
  int day;
  std::string name;
  MapPoint point;
  std::optional<unsigned short> group;
};

std::vector<Command> readCommands(const std::string& path, int days) {
  std::vector<Command> result;
  if(path == "-") return result;
  std::ifstream file(path);
  if(!file) throw std::runtime_error("cannot open actions: " + path);
  std::string line;
  for(int number = 1; std::getline(file, line); ++number) {
    std::istringstream row(line);
    row >> std::ws;
    if(row.eof() || row.peek() == '#') continue;
    Command c{};
    if(!(row >> c.day >> c.name >> c.point.x >> c.point.y) ||
       c.day < 1 || c.day > days || (!result.empty() && c.day < result.back().day))
      throw std::runtime_error("invalid action or day at line " + std::to_string(number));
    if(c.name == "build") {
      int id;
      if(!(row >> id) || id < 0 || id >= NUM_OF_GROUPS)
        throw std::runtime_error("invalid group at line " + std::to_string(number));
      c.group = static_cast<unsigned short>(id);
    } else if(c.name != "query" && c.name != "bulldoze" && c.name != "evacuate" && c.name != "flood") {
      throw std::runtime_error("unknown action: " + c.name);
    }
    std::string extra;
    if(row >> extra) throw std::runtime_error("extra input at line " + std::to_string(number));
    result.push_back(c);
  }
  if(file.bad()) throw std::runtime_error("error reading actions");
  return result;
}

class Observations {
public:
  Observations(const std::filesystem::path& path, const World& world)
    : previous(world.map.len() * world.map.len()), out(path) {
    out.exceptions(std::ios::failbit | std::ios::badbit);
  }

  template<class T>
  void field(int day, const std::string& entity, const char* name, const T& value) {
    out << day << '\t' << entity << '\t' << name << '\t' << value << '\n';
  }

  void snapshot(int day, const World& world) {
    field(day, "world", "time", world.total_time);
    field(day, "world", "money", world.total_money);
    field(day, "world", "people_pool", world.people_pool);
    field(day, "world", "housed_acc", world.stats.population.housed_m.acc);
    field(day, "world", "population_acc", world.stats.population.population_m.acc);
    field(day, "world", "tech", world.tech_level);
    std::size_t index = 0;
    for(const MapTile& tile : world.map) {
      auto& last = previous[index++];
      const auto* building = tile.getConstruction();
      if(!building && last.group == -1) continue;
      const int group = building ? building->getConstructionGroup()->group : -1;
      const bool replaced = last.group != group;
      if(!replaced && building->commodityCount == last.goods) continue;
      const std::string entity = "building(" + std::to_string(tile.point.x) +
        "," + std::to_string(tile.point.y) + ")";
      if(replaced) field(day, entity, "group", group);
      if(building) {
        for(Commodity c = STUFF_INIT; c < STUFF_COUNT; ++c) {
          if(replaced || last.goods[c] != building->commodityCount[c])
            field(day, entity, commodityStandardName(c), building->commodityCount[c]);
        }
        last.goods = building->commodityCount;
      }
      last.group = group;
    }
  }

  void close() { out.close(); }

private:
  struct Building {
    int group = -1;
    std::array<int, STUFF_COUNT> goods{};
  };
  std::vector<Building> previous;
  std::ofstream out;
};

void execute(World& world, Observations& out, const Command& command, std::size_t index) {
  UserOperation op;
  if(command.group) {
    op.action = UserOperation::ACTION_BUILD;
    op.constructionGroup = ConstructionGroup::getConstructionGroup(*command.group);
    if(!op.constructionGroup) throw std::runtime_error("unregistered building group");
  } else if(command.name == "bulldoze") op.action = UserOperation::ACTION_BULLDOZE;
  else if(command.name == "evacuate") op.action = UserOperation::ACTION_EVACUATE;
  else if(command.name == "flood") op.action = UserOperation::ACTION_FLOOD;
  const auto p = command.point;
  const std::string entity = "action[" + std::to_string(index) + "]." + command.name +
    "(" + std::to_string(p.x) + "," + std::to_string(p.y) + ")";
  Message::ptr reason;
  const bool allowed = op.isAllowedHere(world, p, reason);
  out.field(command.day, entity, "allowed", allowed);
  if(!allowed) out.field(command.day, entity, "reason", describeEvent(reason));
  if(world.map.is_visible(p)) out.field(command.day, entity, "quote", op.cost(world, p));
  const int before = world.total_money;
  try {
    op.execute(world, p);
    out.field(command.day, entity, "result", "ok");
  } catch(const Message::Exception& error) {
    out.field(command.day, entity, "result", describeEvent(error.getMessage()));
  }
  out.field(command.day, entity, "money_delta", world.total_money - before);
}
}

int main(int argc, char** argv) {
  if(argc != 8) {
    std::cerr << "usage: core_replay record|replay TAPE SCENARIO traffic|reduced|no-traffic DAYS ACTIONS|- OUTPUT_DIR\n";
    return 2;
  }
  try {
    std::string mode(argv[1]), profile(argv[4]), count(argv[5]);
    int days;
    auto parsed = std::from_chars(count.data(), count.data() + count.size(), days);
    if(parsed.ec != std::errc() || parsed.ptr != count.data() + count.size() || days < 0)
      throw std::runtime_error("invalid day count");
    if(profile != "traffic" && profile != "reduced" && profile != "no-traffic") throw std::runtime_error("invalid traffic profile");
    const auto commands = readCommands(argv[6], days);
    std::filesystem::path dir(argv[7]);
    std::filesystem::create_directories(dir);
    std::ifstream input;
    std::ofstream output;
    std::optional<SimulationTrace> trace;
    if(mode == "record") {
      output.open(argv[2]);
      output.exceptions(std::ios::badbit | std::ios::failbit);
      trace.emplace(output);
    } else if(mode == "replay") {
      input.open(argv[2]);
      if(!input) throw std::runtime_error("cannot open decision tape");
      trace.emplace(input);
    } else throw std::runtime_error("invalid mode");

    std::srand(714);
    BasicUrbg::get().seed(714);
    auto world = World::load(argv[3]);
    if(profile == "traffic") world->vehicleTraffic = World::VehicleTraffic::Normal;
    else if(profile == "reduced") world->vehicleTraffic = World::VehicleTraffic::Reduced;
    else world->vehicleTraffic = World::VehicleTraffic::Disabled;
    Observations out(dir / "state.tsv", *world);
    out.snapshot(0, *world);
    world->save(dir / "day-0.gz");
    std::size_t next = 0;
    for(int day = 0; day < days;) {
      ++day;
      while(next < commands.size() && commands[next].day == day) {
        execute(*world, out, commands[next], next);
        ++next;
      }
      world->do_animate(1000UL * day);
      world->do_time_step();
      for(int i = 0; auto message = world->popMessage(); ++i)
        out.field(day, "messages", std::to_string(i).c_str(), describeEvent(message));
      out.snapshot(day, *world);
      if(day == 1 || day == NUMOF_DAYS_IN_MONTH || day == days)
        world->save(dir / ("day-" + std::to_string(day) + ".gz"));
    }
    trace->finish();
    out.close();
    if(output.is_open()) output.close();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
