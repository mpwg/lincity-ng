// C++ oracle for the v1 core slice. See README.md for the case format.
#include "lincity/commodities.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/solar_power.hpp"
#include "lincity/modules/substation.hpp"
#include "lincity/stats.hpp"
#include "lincity/world.hpp"

#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>

namespace {
int number(std::istringstream& row) {
  int value;
  if (!(row >> value)) throw std::runtime_error("expected integer");
  return value;
}

void names() {
  for (Commodity c = STUFF_INIT; c < STUFF_COUNT; ++c) {
    auto name = commodityStandardName(c);
    std::cout << name << ' ' << commodityFromStandardName(name) << '\n';
  }
  for (auto invalid : {"Food", "", "unknown"})
    std::cout << commodityStandardName(commodityFromStandardName(invalid)) << '\n';
}

void inventory(std::istringstream& row) {
  Stats::Inventory<> value{number(row), number(row)};
  auto print = [&] { std::cout << value.amount << ' ' << value.capacity << ' ' << value.percent() << '\n'; };
  print();
  value += Stats::Inventory<>{number(row), number(row)};
  print();
  value -= Stats::Inventory<>{number(row), number(row)};
  print();
}

void stats(std::istringstream& row) {
  Stats s;
  auto& pop = s.population;
  auto& food = s.inventory[STUFF_FOOD];
  pop.population_m += number(row);
  pop.housed_m += number(row);
  food.amount += number(row);
  food.capacity += number(row);
  s.income.goods_tax += number(row);
  s.expenses.transport += number(row);
  pop.births_m += number(row);
  pop.unemployed_history = number(row);
  auto print = [&] {
    // Compare the exact float result after C++'s double-precision multiply.
    uint32_t decay;
    static_assert(sizeof(decay) == sizeof(pop.unemployed_history));
    std::memcpy(&decay, &pop.unemployed_history, sizeof(decay));
    const auto& history = s.history.inventory[STUFF_FOOD];
    std::cout << pop.population_m.acc << ' ' << pop.population_m.stat << ' '
              << s.history.pop.size() << ' ' << s.history.pop[0] << ' ' << s.history.pop[1]
              << ' ' << s.history.ppool[0] << ' ' << food.amount.acc << ' ' << food.capacity.acc
              << ' ' << history.size() << ' ' << history[0].amount << ' ' << history[0].capacity
              << ' ' << history[1].percent() << ' ' << s.income.goods_tax.acc << ' '
              << s.income.goods_tax.stat << ' ' << s.expenses.transport.acc << ' '
              << s.expenses.transport.stat << ' ' << pop.births_m.acc << ' '
              << pop.births_m.stat << ' ' << decay << '\n';
  };
  std::string sequence;
  if (!(row >> sequence)) throw std::runtime_error("expected boundary sequence");
  for (auto boundary : sequence) {
    switch (boundary) {
      case 'd': s.daily(); break;
      case 'm': s.monthly(); break;
      case 'y': s.yearly(); break;
      default: throw std::runtime_error("unknown statistics boundary");
    }
    print();
  }
}

void chain(std::istringstream& row) {
  World world(32);
  world.tech_level = MAX_TECH_LEVEL;
  const MapPoint source(10, 10), sink = source.e(GROUP_SOLAR_POWER_SIZE);
  solarPowerConstructionGroup.placeItem(world, source);
  substationConstructionGroup.placeItem(world, sink);
  auto& solar = *world.map(source)->getConstruction();
  auto* station = world.map(sink)->getConstruction();
  solar.levelStuff(STUFF_LABOR, number(row));
  solar.levelStuff(STUFF_HIVOLT, number(row));
  station->levelStuff(STUFF_HIVOLT, number(row));
  station->levelStuff(STUFF_LOVOLT, number(row));
  int ticks = number(row);
  int remove_at = number(row);
  for (int tick = 0; tick <= ticks; ++tick) {
    if (tick) {
      world.total_time = tick;
      if (tick == remove_at) world.bulldozeArea(sink);
      station = world.map(sink)->getConstruction();
      solar.trade();
      solar.update();
      if (station) {
        station->trade();
        station->update();
      }
    }
    std::cout << tick << ' ' << solar.commodityCount[STUFF_LABOR] << ' '
              << solar.commodityCount[STUFF_HIVOLT] << ' ';
    if (station) std::cout << station->commodityCount[STUFF_HIVOLT] << ' '
                           << station->commodityCount[STUFF_LOVOLT];
    else std::cout << "removed";
    std::cout << '\n';
  }
}
}

int main(int argc, char** argv) {
  if (argc != 2) return 2;
  std::ifstream input(argv[1]);
  std::string line;
  if (!std::getline(input, line) || line != "version 1") return 2;
  std::cout << line << '\n';
  try {
    while (std::getline(input, line)) {
      std::istringstream row(line);
      std::string command;
      row >> command;
      std::cout << line << '\n';
      if (command == "names") names();
      else if (command == "inventory") inventory(row);
      else if (command == "stats") stats(row);
      else if (command == "chain") chain(row);
      else throw std::runtime_error("unknown case");
      if (row >> command) throw std::runtime_error("unexpected argument");
    }
  } catch (const std::exception& error) {
    std::cerr << line << ": " << error.what() << '\n';
    return 1;
  }
  return input.bad() ? 1 : 0;
}
