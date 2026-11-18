// Each row starts a fresh world, places a single building and visits it daily.
// Columns: group, technology, day, days, cash, coal/ore deposit, ten stocks.
#include "lincity/lintypes.hpp"
#include "lincity/world.hpp"
#include <fstream>
#include <iostream>
#include <sstream>

int main(int argc, char** argv) {
  if(argc != 2) return 2;
  std::ifstream input(argv[1]);
  if(!input) return 2;

  const MapPoint site(10, 10);
  std::string line;
  while(std::getline(input, line)) {
    World world(32);
    int id, firstDay, days;
    auto& tile = *world.map(site);
    for(int y = 0; y < 32; ++y)
      for(int x = 0; x < 32; ++x) {
        auto& t = *world.map(MapPoint(x, y));
        t.coal_reserve = t.ore_reserve = t.pollution = 0;
      }
    std::istringstream row(line);
    if(!(row >> id >> world.tech_level >> firstDay >> days >> world.total_money
      >> tile.coal_reserve >> tile.ore_reserve)) return 2;
    world.stats.sustainability.mining_flag = true;
    auto group = ConstructionGroup::getConstructionGroup(id);
    if(!group) return 2;
    group->placeItem(world, site);
    auto* building = tile.getConstruction();
    for(auto& amount : building->commodityCount)
      if(!(row >> amount)) return 2;
    std::cout << line << '\n';
    for(int day = firstDay; day < firstDay + days; ++day) {
      world.total_time = day;
      if(building) building->update();
      building = tile.getConstruction();
      std::cout << day << ' ' << world.total_money << ' ' << world.stats.expenses.recycle.acc
        << ' ' << world.stats.sustainability.mining_flag << ' ' << tile.pollution
        << ' ' << tile.coal_reserve << ' ' << tile.ore_reserve;
      if(building) {
        std::cout << ' ' << bool(building->flags & FLAG_EVACUATE);
        for(const auto& series : {building->commodityCount, building->commodityProd,
            building->commodityProdPrev})
          for(auto amount : series) std::cout << ' ' << amount;
      } else std::cout << " removed";
      std::cout << '\n';
    }
  }
}
