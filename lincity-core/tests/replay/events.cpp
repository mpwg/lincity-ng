#include "events.hpp"

#include "lincity/lintypes.hpp"
#include <stdexcept>

namespace {
std::string location(MapPoint p) {
  return " x=" + std::to_string(p.x) + " y=" + std::to_string(p.y);
}

std::string group(const ConstructionGroup& g) {
  return " group=" + std::to_string(g.group);
}
}

std::string describeEvent(const Message::ptr& message) {
  if(!message) return "none";
  if(auto m = dynamic_message_cast<CannotBuildHereMessage>(message))
    return "cannot-build-here" + location(m->getPoint()) + group(m->getGroup()) +
      " reason=" + describeEvent(m->getReason());
  if(auto m = dynamic_message_cast<CannotBuildMessage>(message))
    return "cannot-build" + group(m->getGroup()) + " reason=" + describeEvent(m->getReason());
  if(auto m = dynamic_message_cast<CannotBulldozeThisMessage>(message)) {
    std::string type = "cannot-bulldoze";
    if(dynamic_message_cast<CannotBulldozeIncompleteMonumentMessage>(message))
      type = "incomplete-monument";
    else if(dynamic_message_cast<CannotBulldozeNonemptyTipMessage>(message))
      type = "nonempty-tip";
    else if(dynamic_message_cast<CannotBulldozeThisEverMessage>(message))
      type = "protected-building";
    return type + location(m->getPoint()) + group(m->getGroup());
  }
  if(auto m = dynamic_message_cast<CannotEvacuateThisMessage>(message))
    return "cannot-evacuate" + location(m->getPoint()) + group(m->getGroup());
  if(auto m = dynamic_message_cast<CannotEvacuateNothingMessage>(message))
    return "nothing-to-evacuate" + location(m->getPoint());
  if(auto m = dynamic_message_cast<FireStartedMessage>(message))
    return "fire-started" + location(m->getPoint()) + group(m->getGroup());
  if(auto m = dynamic_message_cast<OutOfMoneyMessage>(message))
    return std::string("out-of-money credit-exhausted=") + (m->isOutOfCredit() ? "1" : "0");
  if(dynamic_message_cast<NoOreMessage>(message)) return "no-ore";
  if(auto m = dynamic_message_cast<DesertHereMessage>(message))
    return "desert-here" + location(m->getPoint());
  if(dynamic_message_cast<PortRequiresRiverMessage>(message)) return "port-requires-river";
  if(dynamic_message_cast<NotEnoughStudentsMessage>(message)) return "not-enough-students";
  if(auto m = dynamic_message_cast<OutsideMapMessage>(message))
    return "outside-map" + location(m->getPoint());
  if(auto m = dynamic_message_cast<NotEnoughTechMessage>(message))
    return "not-enough-tech current=" + std::to_string(m->getCurrentTech()) +
      " required=" + std::to_string(m->getRequiredTech());
  if(auto m = dynamic_message_cast<SpaceOccupiedMessage>(message))
    return "space-occupied" + location(m->getPoint());
  if(auto m = dynamic_message_cast<SustainableEconomyMessage>(message))
    return "sustainable-economy time=" + std::to_string(m->getTime());
  if(auto m = dynamic_message_cast<NoPeopleLeftMessage>(message))
    return "no-people-left time=" + std::to_string(m->getTime());
  if(auto m = dynamic_message_cast<RocketReadyMessage>(message))
    return "rocket-ready" + location(m->getPoint());
  if(auto m = dynamic_message_cast<RocketResultMessage>(message)) {
    switch(m->getResult()) {
    case RocketResultMessage::LaunchResult::FAIL: return "rocket-failed" + location(m->getPoint());
    case RocketResultMessage::LaunchResult::SUCCESS: return "rocket-success" + location(m->getPoint());
    case RocketResultMessage::LaunchResult::EVAC: return "rocket-evacuated" + location(m->getPoint());
    case RocketResultMessage::LaunchResult::EVAC_WIN: return "rocket-won" + location(m->getPoint());
    }
  }
  throw std::runtime_error("unhandled simulation event: " + message->str());
}
