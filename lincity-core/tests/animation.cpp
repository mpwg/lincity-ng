// Animation state advances on the caller's clock, even without a renderer.
#include "lincity/modules/rocket_pad.hpp"
#include "lincity/modules/residence.hpp"
#include "lincity/modules/all_modules.hpp"
#include "util/simulation_trace.hpp"
#include "lincity/world.hpp"

#include <iostream>
#include <sstream>
#include <stdexcept>

namespace {
void require(bool condition, const char* message) {
  if(!condition) throw std::runtime_error(message);
}

void checkFrameCounts() {
  World first(32);
  const auto& groups = ResourceGroup::resMap;
  const auto* rocket = groups.at("Rocket");
  require(rocket->frameCount == 8, "rocket animation length");
  require(groups.at("Bluecar")->frameCount == 16, "vehicle animation length");
  require(groups.at("Fire")->frameCount == 5, "fire animation length");
  require(groups.at("BlackSmoke")->frameCount == 11, "coal smoke animation length");
  require(groups.at("GraySmoke")->frameCount == 10, "factory smoke animation length");
  World second(32);
  require(rocket == groups.at("Rocket"), "second world replaced resource metadata");
}

void checkRocketTiming() {
  World world(32);
  const MapPoint site(8, 8);
  world.tech_level = MAX_TECH_LEVEL;
  world.total_money = 1000000;
  world.rockets_launched = 10; // full technology and experience guarantee success
  rocketPadConstructionGroup.placeItem(world, site);
  auto& rocket = dynamic_cast<RocketPad&>(*world.map(site)->getConstruction());
  rocket.steps = ROCKET_PAD_STEPS - 1;
  rocket.levelStuff(STUFF_LABOR, ROCKET_PAD_LABOR);
  rocket.levelStuff(STUFF_GOODS, ROCKET_PAD_GOODS);
  rocket.levelStuff(STUFF_STEEL, ROCKET_PAD_STEEL);
  world.do_time_step();
  require(rocket.stage == RocketPad::AWAITING, "pad failed to finish building");
  world.do_animate(0);
  require(rocket.getFrame().frame == 4, "ready frame");
  require(world.launchRocket(site), "launch command rejected");

  const unsigned long start = 100;
  const unsigned long deadline = start + ANIM_THRESHOLD(ROCKET_ANIMATION_SPEED);
  world.do_animate(start);
  require(rocket.getFrame().frame == 5, "takeoff frame");
  world.do_time_step();
  world.do_animate(deadline - 1);
  require(rocket.stage == RocketPad::LAUNCHING && rocket.getFrame().frame == 5,
    "takeoff finished before its animation deadline");
  world.do_animate(deadline);
  require(rocket.stage == RocketPad::LAUNCH && rocket.getFrame().frame == 6,
    "takeoff did not finish at its animation deadline");
  require(world.rockets_launched == 10, "animation calculated a launch result");

  world.do_time_step();
  require(rocket.stage == RocketPad::DONE && world.rockets_launched == 11,
    "simulation tick failed to resolve launch");
  bool success = false;
  while(auto message = world.popMessage()) {
    if(auto result = dynamic_message_cast<RocketResultMessage>(message))
      success = result->getPoint() == site &&
        result->getResult() == RocketResultMessage::LaunchResult::SUCCESS;
  }
  require(success, "successful launch result missing");
  world.do_animate(deadline);
  require(rocket.getFrame().frame == 7, "completed rocket frame");
  world.do_time_step();
  require(world.rockets_launched == 11, "launch result applied twice");
}
void checkEvacuation(int pool, int residents, bool ended,
                     RocketResultMessage::LaunchResult expected) {
  World world(32);
  const MapPoint site(8, 8), home(18, 18);
  world.tech_level = MAX_TECH_LEVEL;
  world.rockets_launched = 10;
  world.rockets_launched_success = 5;
  world.people_pool = pool;
  world.gameEnd = ended;
  rocketPadConstructionGroup.placeItem(world, site);
  residenceLLConstructionGroup.placeItem(world, home);
  auto& rocket = dynamic_cast<RocketPad&>(*world.map(site)->getConstruction());
  auto& residence = dynamic_cast<Residence&>(*world.map(home)->getConstruction());
  residence.local_population = residents;
  rocket.stage = RocketPad::LAUNCH;
  rocket.update();
  const auto result = dynamic_message_cast<RocketResultMessage>(world.popMessage());
  require(result && result->getPoint() == site && result->getResult() == expected,
          "incorrect evacuation result");
  require(world.rockets_launched_success == 6 && rocket.stage == RocketPad::DONE,
          "evacuation did not complete the launch");
  const int evacuated = std::min(1000, pool + residents);
  require(world.stats.population.evacuated_t == evacuated,
          "incorrect number of evacuated people");
  require(residence.local_population == residents - std::max(0, evacuated - pool),
          "incorrect number of residents after evacuation");
  require(world.people_pool == std::max(pool - evacuated, 0), "incorrect remaining people pool");
  require(world.gameEnd == (ended || expected == RocketResultMessage::LaunchResult::EVAC_WIN),
          "incorrect end-of-game state");
  require(!world.popMessage(), "duplicate evacuation result");
}

void checkCrash() {
  World world(32);
  const MapPoint site(8, 8), target(18, 18);
  rocketPadConstructionGroup.placeItem(world, site);
  blacksmithConstructionGroup.placeItem(world, target);
  auto& rocket = dynamic_cast<RocketPad&>(*world.map(site)->getConstruction());
  rocket.stage = RocketPad::LAUNCH;
  world.rockets_launched_success = 4;
  std::string tape = "LINCITY-SIMULATION-TRACE 1\nS rocket.success 0\n"
    "S rocket.center-x 10\nS rocket.center-y 10\n";
  for(int i = 0; i < 20; ++i) {
    tape += "S rocket.fire-x 0\nS rocket.fire-y 0\n";
    if(i == 0)
      for(auto p : {target, target.e(), target.s(), target.se()})
        tape += "P " + std::to_string(1 + (p.x - target.x) + 2 * (p.y - target.y)) + " " +
          std::to_string(p.x) + " " + std::to_string(p.y) + " " + std::to_string(GROUP_FIRE) + "\n";
  }
  std::istringstream recording(tape + "Z\n");
  SimulationTrace trace(recording);
  rocket.update();
  trace.finish();
  const auto result = dynamic_message_cast<RocketResultMessage>(world.popMessage());
  require(result && result->getResult() == RocketResultMessage::LaunchResult::FAIL,
          "failed launch did not emit its result");
  require(world.rockets_launched_success == 0 && rocket.stage == RocketPad::DONE,
          "failed launch did not reset the success streak");
  for(auto p : {target, target.e(), target.s(), target.se()})
    require(world.map(p)->getGroup() == GROUP_FIRE, "crash did not ignite the footprint");
}

}

int main() {
  try {
    checkFrameCounts();
    checkRocketTiming();
    using Result = RocketResultMessage::LaunchResult;
    checkEvacuation(998, 5, false, Result::EVAC);
    checkEvacuation(2, 1, false, Result::EVAC_WIN);
    checkEvacuation(0, 0, true, Result::EVAC);
    checkCrash();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
