// Animation state advances on the caller's clock, even without a renderer.
#include "lincity/modules/rocket_pad.hpp"
#include "lincity/world.hpp"

#include <iostream>
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
  auto& rocket = dynamic_cast<RocketPad&>(*world.map(site)->construction);
  rocket.steps = ROCKET_PAD_STEPS - 1;
  rocket.levelStuff(STUFF_LABOR, ROCKET_PAD_LABOR);
  rocket.levelStuff(STUFF_GOODS, ROCKET_PAD_GOODS);
  rocket.levelStuff(STUFF_STEEL, ROCKET_PAD_STEEL);
  world.do_time_step();
  require(rocket.stage == RocketPad::AWAITING, "pad failed to finish building");
  world.do_animate(0);
  require(rocket.frameIt->frame == 4, "ready frame");
  require(world.launchRocket(site), "launch command rejected");

  const unsigned long start = 100;
  const unsigned long deadline = start + ANIM_THRESHOLD(ROCKET_ANIMATION_SPEED);
  world.do_animate(start);
  require(rocket.frameIt->frame == 5, "takeoff frame");
  world.do_time_step();
  world.do_animate(deadline - 1);
  require(rocket.stage == RocketPad::LAUNCHING && rocket.frameIt->frame == 5,
    "takeoff finished before its animation deadline");
  world.do_animate(deadline);
  require(rocket.stage == RocketPad::LAUNCH && rocket.frameIt->frame == 6,
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
  require(rocket.frameIt->frame == 7, "completed rocket frame");
  world.do_time_step();
  require(world.rockets_launched == 11, "launch result applied twice");
}
}

int main() {
  try {
    checkFrameCounts();
    checkRocketTiming();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
