// Verify the compatibility view through the entry points used by the desktop.
#include "RustDesktopWorld.hpp"
#include "lincity/ConstructionReport.hpp"
#include "lincity/modules/all_modules.hpp"
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

#define CHECK(expr)                                                            \
  do {                                                                         \
    if (!(expr))                                                               \
      throw std::runtime_error(#expr);                                         \
  } while (false)
struct Report final : ConstructionReport {
  std::vector<std::string> labels;
  void addBlank() override {}
  void addLabel(const std::string &label) override { labels.push_back(label); }
  void addLabeledText(const std::string &, const std::string &) override {}
  void addInteger(const std::string &, int) override {}
  void addDecimal(const std::string &, double) override {}
  void addPercentage(const std::string &, double) override {}
  void addQuantity(const std::string &, int, int) override {}
  void addPrefixedQuantity(const std::string &, const std::string &, int,
                           int) override {}
};
int main(int argc, char **argv) {
  if (argc != 2)
    return 1;
  try {
    auto w = DesktopWorld::load(argv[1]);
    CHECK(w->map.len() == 40 && w->total_time == 98 && w->total_money == 10000);
    const std::array terrain{GROUP_BARE, GROUP_DESERT, GROUP_TREE, GROUP_TREE2,
                             GROUP_TREE3};
    for (unsigned i = 0; i < terrain.size(); ++i)
      CHECK(w->map({24 + int(i), 20})->group == terrain[i]);
    const auto &t = *w->map({27, 12});
    CHECK(t.group == GROUP_WATER);
    CHECK(t.is_water() && t.is_river() && t.pollution == 21 &&
          t.ore_reserve == 13 && t.coal_reserve == 8);
    CHECK(w->coal_survey_done && t.ground.altitude == 42 &&
          t.ground.water_alt == 40);
    auto *market =
        dynamic_cast<Market *>(w->map({3, 3})->getReportingConstruction());
    CHECK(market && market->commodityCount[STUFF_FOOD] == 200 &&
          market->commodityRuleCount[STUFF_FOOD].give &&
          !market->commodityRuleCount[STUFF_FOOD].take);
    auto *port =
        dynamic_cast<Port *>(w->map({9, 3})->getReportingConstruction());
    CHECK(port && port->commodityRuleCount[STUFF_GOODS].give &&
          !port->commodityRuleCount[STUFF_GOODS].take);
    CHECK(w->setMarketTrade({2, 2}, STUFF_FOOD, true, false));
    CHECK(w->setPortTrade({8, 2}, STUFF_GOODS, true, false));
    CHECK(w->map({2, 2})->getConstruction() == market &&
          market->commodityRuleCount[STUFF_FOOD].take &&
          !market->commodityRuleCount[STUFF_FOOD].give);
    CHECK(w->map({8, 2})->getConstruction() == port &&
          port->commodityRuleCount[STUFF_GOODS].take &&
          !port->commodityRuleCount[STUFF_GOODS].give);
    auto *rocket =
        dynamic_cast<RocketPad *>(w->map({14, 2})->getConstruction());
    auto *fire = dynamic_cast<Fire *>(w->map({22, 2})->getConstruction());
    CHECK(rocket && rocket->stage == RocketPad::AWAITING && fire &&
          fire->smoking_days == 100);
    CHECK(dynamic_cast<Residence *>(w->map({12, 11})->getConstruction())
              ->local_population == 7);
    auto *station = w->map({2, 11})->getConstruction();
    w->do_animate(1000);
    CHECK(station == w->map({2, 11})->getConstruction() &&
          station->getFrame().resourceGroup->resourceID == "SubstationOn");
    CHECK(fire->getFrame().resourceGroup->resourceID == "FireWasteLand");
    Report report;
    CHECK(renderRustReport(w->engine(), {3, 12}, true, report));
    CHECK(report.labels ==
          std::vector<std::string>({"Power Substation", "Production:"}));
    CHECK(!renderRustReport(w->engine(), {30, 30}, false, report));
    w->do_time_step();
    CHECK(w->total_time == 99 && w->stats.groupCount[GROUP_PORT] == 1);
    CHECK(w->map({3, 3})->getReportingConstruction() == market);
    UserOperation remove;
    remove.action = UserOperation::ACTION_BULLDOZE;
    desktopApply(*w, remove, {3, 3});
    CHECK(!w->map({2, 2})->getConstruction() &&
          !w->map({3, 3})->getReportingConstruction());
    CHECK(w->map({8, 2})->getConstruction() == port);
    CHECK(w->launchRocket({14, 2}));
    CHECK(rocket->stage == RocketPad::LAUNCHING);
    w->do_animate(2000);
    w->do_animate(3000);
    CHECK(rocket->stage == RocketPad::LAUNCH);

  } catch (const std::exception &e) {
    std::cerr << e.what() << '\n';
    return 1;
  }
}
