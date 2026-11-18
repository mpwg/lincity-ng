// Emit the initialized construction registry for the Rust catalog parity test.
#include "lincity/groups.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/world.hpp"
#include <iostream>

int main() {
  World world(32);
  world.tech_level = 783419;
  initializeModules();
  for(unsigned short id = 0; id < NUM_OF_GROUPS; ++id) {
    const auto* g = ConstructionGroup::getConstructionGroup(id);
    if(!g) continue;
    std::cout << id << '\t' << g->resourceID << '\t' << g->name << '\t'
      << g->name_plural << '\t' << g->size << '\t' << g->colour << '\t'
      << g->cost << '\t' << g->cost_mul << '\t' << g->bul_cost << '\t'
      << g->fire_chance << '\t' << g->tech << '\t' << g->range << '\t'
      << g->no_credit << '\t' << g->getCosts(world);
    for(const auto& rule : g->commodityRuleCount)
      std::cout << '\t' << rule.maxload << ',' << rule.take << ',' << rule.give;
    std::cout << '\n';
  }
}
