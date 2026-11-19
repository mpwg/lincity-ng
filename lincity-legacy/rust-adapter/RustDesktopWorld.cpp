// Disposable adapter for the legacy SDL desktop. Rust owns the simulated city.
#include "DesktopWorld.hpp"
#include "lincity/lintypes.hpp"

#include <algorithm>

namespace {
rustcity::Point toRust(MapPoint p) { return {p.x, p.y}; }
MapPoint fromRust(rustcity::Point p) { return {p.x, p.y}; }

std::string pathUtf8(const std::filesystem::path& path) {
  const auto utf8 = path.u8string();
  return std::string(reinterpret_cast<const char*>(utf8.data()), utf8.size());
}

rustcity::ActionKind kind(const UserOperation& op) {
  using K = rustcity::ActionKind;
  switch(op.action) {
    case UserOperation::ACTION_QUERY: return K::Query;
    case UserOperation::ACTION_BUILD: return K::Build;
    case UserOperation::ACTION_BULLDOZE: return K::Bulldoze;
    case UserOperation::ACTION_FLOOD: return K::Flood;
    case UserOperation::ACTION_EVACUATE: return K::Evacuate;
    default: throw std::invalid_argument("unknown operation");
  }
}

unsigned short groupId(const UserOperation& op) {
  return op.constructionGroup ? op.constructionGroup->group : 0;
}

ConstructionGroup& group(unsigned short id) {
  auto* result = ConstructionGroup::getConstructionGroup(id);
  if(!result) throw std::runtime_error("unknown RustCity construction group");
  return *result;
}

Message::ptr failure(const rustcity::ActionResult& r) {
  using F = rustcity::Failure;
  const MapPoint p = fromRust(r.point);
  switch(r.error) {
    case F::None: return {};
    case F::OutsideMap: return OutsideMapMessage::create(p);
    case F::SpaceOccupied: return SpaceOccupiedMessage::create(p);
    case F::NothingToEvacuate: return CannotEvacuateNothingMessage::create(p);
    case F::CannotEvacuate: return CannotEvacuateThisMessage::create(p, group(r.group));
    case F::NoCredit: return OutOfMoneyMessage::create(true);
    case F::OutOfMoney: return OutOfMoneyMessage::create(false);
    case F::NotEnoughTech: return NotEnoughTechMessage::create(r.current_tech, r.required_tech);
    case F::NotEnoughStudents: return NotEnoughStudentsMessage::create();
    case F::DesertHere: return DesertHereMessage::create(p);
    case F::RequiresRiver: return PortRequiresRiverMessage::create();
    case F::NoOre: return NoOreMessage::create();
    case F::NonemptyTip: return CannotBulldozeNonemptyTipMessage::create(p);
    case F::IncompleteMonument: return CannotBulldozeIncompleteMonumentMessage::create(p);
    case F::ProtectedFire: return CannotBulldozeThisEverMessage::create(p, group(r.group));
    default: throw std::runtime_error("RustCity rejected the desktop operation");
  }
}

Message::ptr message(const rustcity::Event& e) {
  using K = rustcity::EventKind;
  using R = RocketResultMessage::LaunchResult;
  const MapPoint p = fromRust(e.point);
  switch(e.kind) {
    case K::None: return {};
    case K::RocketFailed: return RocketResultMessage::create(p, R::FAIL);
    case K::RocketWon: return RocketResultMessage::create(p, R::EVAC_WIN);
    case K::RocketEvacuated: return RocketResultMessage::create(p, R::EVAC);
    case K::RocketReady: return RocketReadyMessage::create(p);
    case K::RocketSuccess: return RocketResultMessage::create(p, R::SUCCESS);
    case K::FireStarted: return FireStartedMessage::create(p, group(e.group));
    case K::OutOfMoney: return OutOfMoneyMessage::create(false);
    case K::NoPeopleLeft: return NoPeopleLeftMessage::create(e.value);
    case K::SustainableEconomy: return SustainableEconomyMessage::create(e.value);
  }
  throw std::runtime_error("unknown RustCity event");
}
}

DesktopWorld::DesktopWorld(rust::Box<rustcity::Engine> e)
  : World(rustcity::side(*e)), engine_(std::move(e)) {
  refresh(true);
}

std::unique_ptr<DesktopWorld> DesktopWorld::load(const std::filesystem::path& path) {
  return std::make_unique<DesktopWorld>(rustcity::load(pathUtf8(path)));
}

void DesktopWorld::save(const std::filesystem::path& path) const {
  rustcity::save(engine(), pathUtf8(path));
}

std::unique_ptr<DesktopWorld> generateDesktopCity(DesktopClimate climate, city_settings* settings, int side) {
  using C = rustcity::Climate;
  C value;
  switch(climate) {
    case DesktopClimate::River: value = C::River; break;
    case DesktopClimate::Desert: value = C::Desert; break;
    case DesktopClimate::Temperate: value = C::Temperate; break;
    case DesktopClimate::Swamp: value = C::Swamp; break;
    default: throw std::invalid_argument("unknown terrain climate");
  }
  return std::make_unique<DesktopWorld>(rustcity::generate(side, value, settings->with_village, !settings->without_trees));
}

void DesktopWorld::refresh(bool simulationChanged) {
  const int oldBalance = total_money;
  syncRustView(*this, engine(), simulationChanged, projection_);
  while(auto event = message(rustcity::pop_event(engine())))
    pushMessage(std::move(event));
  if(simulationChanged) setUpdated(Updatable::MAP);
  if(total_money != oldBalance) setUpdated(Updatable::MONEY);
}
void DesktopWorld::do_time_step() {
  using T = rustcity::Traffic;
  rustcity::set_traffic(engine(), vehicleTraffic == VehicleTraffic::Normal ? T::Normal :
    vehicleTraffic == VehicleTraffic::Reduced ? T::Reduced : T::Disabled);
  rustcity::tick(engine());
  refresh(true);
}
void DesktopWorld::do_animate(unsigned long ms) {
  rustcity::animate(engine(), ms);
  refresh(false);
}
bool DesktopWorld::setMarketTrade(MapPoint p, Commodity c, bool buy, bool sell) {
  const bool changed = rustcity::market_trade(engine(), toRust(p), c, buy, sell);
  if(changed) refresh(true);
  return changed;
}
bool DesktopWorld::setPortTrade(MapPoint p, Commodity c, bool buy, bool sell) {
  const bool changed = rustcity::port_trade(engine(), toRust(p), c, buy, sell);
  if(changed) refresh(true);
  return changed;
}
bool DesktopWorld::launchRocket(MapPoint p) {
  const bool changed = rustcity::launch(engine(), toRust(p));
  if(changed) refresh(true);
  return changed;
}
void DesktopWorld::do_coal_survey() {
  if(auto why = failure(rustcity::survey_coal(engine()))) why->throwEx();
  refresh(true);
}
void desktopApply(DesktopWorld& w, const UserOperation& op, MapPoint p) {
  if(auto why = failure(rustcity::apply(w.engine(), kind(op), groupId(op), toRust(p))))
    why->throwEx();
  w.refresh(true);
}
rust::Vec<rustcity::BatchActionResult> desktopPreviewBatch(
  const DesktopWorld& w, const UserOperation& op, rust::Slice<const rustcity::Point> points
) {
  return rustcity::preview_batch(w.engine(), kind(op), groupId(op), points);
}
rust::Vec<rustcity::BatchActionResult> desktopApplyBatch(
  DesktopWorld& w, const UserOperation& op, rust::Slice<const rustcity::Point> points
) {
  auto results = rustcity::apply_batch(w.engine(), kind(op), groupId(op), points);
  if(std::any_of(results.begin(), results.end(), [](const auto& result) {
    return result.action.error == rustcity::Failure::None || result.action.error == rustcity::Failure::DecisionTrace;
  })) w.refresh(true);
  return results;
}
Message::ptr desktopActionMessage(const rustcity::ActionResult& result) {
  return failure(result);
}
bool desktopPreview(const DesktopWorld& w, const UserOperation& op, MapPoint p, Message::ptr& why) {
  why = failure(rustcity::preview(w.engine(), kind(op), groupId(op), toRust(p)));
  return !why;
}
bool desktopAllowed(const DesktopWorld& w, const UserOperation& op, Message::ptr& why) {
  why = op.action == UserOperation::ACTION_BUILD ? failure(rustcity::can_build(w.engine(), groupId(op))) : nullptr;
  return !why && op.action != UserOperation::ACTION_UNKNOWN;
}
ConstructionGroup* desktopBuildGroup(const DesktopWorld& w, const UserOperation& op) {
  return ConstructionGroup::getConstructionGroup(rustcity::can_build(w.engine(), groupId(op)).group);
}
int desktopCost(const DesktopWorld& w, const UserOperation& op) {
  if(op.action == UserOperation::ACTION_BUILD)
    return rustcity::can_build(w.engine(), groupId(op)).cost;
  // The old toolbar uses the border tile as its default terrain for price labels.
  return op.cost(w, {});
}
int desktopCost(const DesktopWorld& w, const UserOperation& op, MapPoint p) {
  return rustcity::quote(w.engine(), kind(op), groupId(op), toRust(p)).cost;
}
void desktopFocus(DesktopWorld& w, MapPoint p) {
  rustcity::set_focus(w.engine(), toRust(p));
  w.map.recentPoint = p;
}
void desktopHistory(DesktopWorld& w, std::size_t months) {
  rustcity::ensure_history(w.engine(), months);
  w.stats.ensurePopulationHistory(months);
}
