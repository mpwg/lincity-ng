// Record the public report rows and installed production limits for each group.
#include "lincity/ConstructionReport.hpp"
#include "lincity/groups.hpp"
#include "lincity/lintypes.hpp"
#include "lincity/modules/all_modules.hpp"
#include "lincity/world.hpp"
#include "util/simulation_trace.hpp"
#include <iostream>
#include <sstream>
#include <string>

struct Rows final : ConstructionReport {
  void addBlank() override { std::cout << "blank\n"; }
  void addLabel(const std::string &k) override {
    std::cout << "label\t" << k << '\n';
  }
  void addLabeledText(const std::string &k, const std::string &v) override {
    std::cout << "text\t" << k << '\t' << v << '\n';
  }
  void addInteger(const std::string &k, int v) override {
    std::cout << "int\t" << k << '\t' << v << '\n';
  }
  void addDecimal(const std::string &k, double v) override {
    std::cout << "decimal\t" << k << '\t' << v << '\n';
  }
  void addPercentage(const std::string &k, double v) override {
    std::cout << "percent\t" << k << '\t' << v << '\n';
  }
  void addQuantity(const std::string &k, int v, int max) override {
    std::cout << "qty\t" << k << '\t' << v << '\t' << max << '\n';
  }
  void addPrefixedQuantity(const std::string &p, const std::string &k, int v,
                           int max) override {
    std::cout << "pref\t" << p << '\t' << k << '\t' << v << '\t' << max << '\n';
  }
};
void printSprites(World &w, MapPoint p, unsigned id, int phase) {
  std::cout << id << ' ' << phase;
  for (auto &f : *w.map(p)->getFrames())
    if (f.frame >= 0)
      std::cout << ' ' << f.resourceGroup->resourceID << ',' << f.frame << ','
                << f.move_x << ',' << f.move_y;
  std::cout << '\n';
}

int main(int argc, char **argv) {
  const bool stocked = argc > 1 && std::string(argv[1]) == "stocked";
  const bool activeSprites =
      argc > 1 && std::string(argv[1]) == "active_sprites";
  const bool working =
      activeSprites || (argc > 1 && std::string(argv[1]) == "working");
  initializeModules();
  for (unsigned short id = 1; id < NUM_OF_GROUPS; ++id) {
    auto *g = ConstructionGroup::getConstructionGroup(id);
    if (!g)
      continue;
    if (working &&
        !(id == GROUP_SOLAR_POWER || id == GROUP_SUBSTATION ||
          id == GROUP_COAL_POWER || id == GROUP_INDUSTRY_L ||
          id == GROUP_INDUSTRY_H || id == GROUP_TIP || id == GROUP_SCHOOL ||
          id == GROUP_BLACKSMITH || id == GROUP_WATERWELL))
      continue;
    World w(40);
    w.tech_level = stocked ? 1750000 : 0;
    auto p = MapPoint(12, 12);
    if (id == GROUP_TRACK_BRIDGE || id == GROUP_ROAD_BRIDGE ||
        id == GROUP_RAIL_BRIDGE)
      w.map(p)->setTerrain(GROUP_WATER);
    Construction::placeOwned(
        std::unique_ptr<Construction>(g->createConstruction(w)), p);
    auto *b = w.map(p)->getConstruction();
    if (argc > 1 && std::string(argv[1]) == "sprites") {
      w.map.connect_transport(1, 1, 38, 38);
      for (int phase = 0; phase < 2; ++phase) {
        if (phase) {
          for (Commodity c = STUFF_INIT; c < STUFF_COUNT; c++)
            b->commodityCount[c] = g->commodityRuleCount[c].maxload * 3 / 4;
          b->animate(1000);
        }
        printSprites(w, p, id, phase);
      }
    } else if (argc > 1 && std::string(argv[1]) == "limits") {
      std::cout << id;
      for (auto v : b->commodityMaxProd)
        std::cout << ' ' << v;
      for (auto v : b->commodityMaxCons)
        std::cout << ' ' << v;
      std::cout << '\n';
    } else {
      if (stocked) {
        b->flags |= FLAG_EVACUATE;
        for (Commodity c = STUFF_INIT; c < STUFF_COUNT; c++) {
          b->commodityCount[c] = g->commodityRuleCount[c].maxload / 3;
          b->commodityProdPrev[c] = b->commodityMaxCons[c]
                                        ? -b->commodityMaxCons[c] / 3
                                        : b->commodityMaxProd[c] / 2;
        }
      }
      if (working) {
        for (int day = 1; day <= 100; ++day) {
          w.total_time = day;
          for (Commodity c = STUFF_INIT; c < STUFF_COUNT; c++)
            b->commodityCount[c] = g->commodityRuleCount[c].maxload / 2;
          b->update();
        }
      }
      if (activeSprites) {
        std::ostringstream draws;
        draws << "LINCITY-SIMULATION-TRACE 1\n";
        int count = id == GROUP_COAL_POWER ? 8 : id == GROUP_INDUSTRY_L ? 2 : 0;
        for (int i = 0; i < count; ++i)
          draws << "C " << i + 5 << '\n';
        draws << "Z\n";
        std::istringstream tape(draws.str());
        SimulationTrace trace(tape);
        b->animate(1000);
        trace.finish();
        printSprites(w, p, id, 1);
        continue;
      }
      Rows r;
      for (bool production : {false, true}) {
        std::cout << "report\t" << id << '\t' << production << '\n';
        b->report(r, production);
      }
    }
  }
}
