// SPDX-License-Identifier: GPL-2.0-or-later
#pragma once

#include <cstdint>
#include <iosfwd>
#include <set>
#include <unordered_map>
#include <vector>

class Construction;

// A trace covers one simulation on the calling thread. Attach before loading
// the world so that placement IDs and constructor-time RNG draws are captured.
class SimulationTrace {
public:
  explicit SimulationTrace(std::ostream& recording);
  explicit SimulationTrace(std::istream& replay);
  SimulationTrace(const SimulationTrace&) = delete;
  SimulationTrace& operator=(const SimulationTrace&) = delete;
  ~SimulationTrace();
  void finish();

  static SimulationTrace* current() { return active; }
  bool replaying() const { return input != nullptr; }

  int cRandom(int value);
  int sample(const char* site, int value, int lower, int upper);
  void placed(const Construction* building, int x, int y, unsigned group);

  using Order = std::vector<std::set<Construction*>::iterator>;
  void visitOrder(char phase, std::uint64_t time, Order& order);

private:
  void expect(char event);
  void checkOutput() const;
  [[noreturn]] void mismatch(const char* detail) const;

  static thread_local SimulationTrace* active;
  std::ostream* output = nullptr;
  std::istream* input = nullptr;
  std::uint64_t events = 0;
  std::uint64_t nextID = 0;
  std::unordered_map<const Construction*, std::uint64_t> ids;
};

// Traversals whose native order is address-dependent use the original set
// directly unless a trace is attached.
template<class Visitor>
void visitConstructions(const std::set<Construction*>& buildings, char phase,
                        std::uint64_t time, Visitor visit) {
  if(auto trace = SimulationTrace::current()) {
    SimulationTrace::Order order;
    order.reserve(buildings.size());
    for(auto it = buildings.begin(); it != buildings.end(); ++it)
      order.push_back(it);
    trace->visitOrder(phase, time, order);
    for(auto it : order)
      if(!visit(*it)) break;
  } else {
    for(auto building : buildings)
      if(!visit(building)) break;
  }
}
