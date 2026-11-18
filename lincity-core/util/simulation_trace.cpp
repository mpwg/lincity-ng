// SPDX-License-Identifier: GPL-2.0-or-later
#include "simulation_trace.hpp"

#include <cstdlib>
#include <istream>
#include <ostream>
#include <stdexcept>
#include <string>

namespace {
constexpr char signature[] = "LINCITY-SIMULATION-TRACE 1";
}

thread_local SimulationTrace* SimulationTrace::active = nullptr;

SimulationTrace::SimulationTrace(std::ostream& recording) : output(&recording) {
  if(active) throw std::logic_error("a simulation trace is already attached");
  recording << signature << '\n';
  checkOutput();
  active = this;
}

SimulationTrace::SimulationTrace(std::istream& replay) : input(&replay) {
  if(active) throw std::logic_error("a simulation trace is already attached");
  std::string header;
  if(!std::getline(replay, header) || header != signature)
    mismatch("invalid header");
  active = this;
}

SimulationTrace::~SimulationTrace() {
  if(active == this) active = nullptr;
}

void SimulationTrace::mismatch(const char* detail) const {
  throw std::runtime_error("simulation trace event " + std::to_string(events)
    + ": " + detail);
}

void SimulationTrace::checkOutput() const {
  if(!*output) mismatch("write failed");
}

void SimulationTrace::expect(char event) {
  char found;
  if(!(*input >> found) || found != event)
    mismatch("unexpected event kind or end of input");
}

void SimulationTrace::finish() {
  if(active != this) throw std::logic_error("simulation trace is inactive");
  if(replaying()) {
    expect('Z');
    *input >> std::ws;
    if(!input->eof() || input->bad()) mismatch("trailing data or read failure");
  } else {
    *output << "Z\n" << std::flush;
    checkOutput();
  }
  active = nullptr;
}

int SimulationTrace::cRandom(int value) {
  ++events;
  if(replaying()) {
    expect('C');
    if(!(*input >> value) || value < 0 || value > RAND_MAX)
      mismatch("random draw outside generator range");
  } else {
    *output << "C " << value << '\n';
    checkOutput();
  }
  return value;
}

int SimulationTrace::sample(const char* site, int value, int lower, int upper) {
  ++events;
  if(replaying()) {
    expect('S');
    std::string recordedSite;
    if(!(*input >> recordedSite) || recordedSite != site)
      mismatch("unexpected distribution site");
    if(!(*input >> value) || value < lower || value > upper)
      mismatch("distribution outcome out of range");
  } else {
    *output << "S " << site << ' ' << value << '\n';
    checkOutput();
  }
  return value;
}

void SimulationTrace::placed(const Construction* building, int x, int y, unsigned group) {
  ++events;
  auto id = ++nextID;
  if(replaying()) {
    expect('P');
    std::uint64_t expectedID;
    int expectedX, expectedY;
    unsigned expectedGroup;
    if(!(*input >> expectedID >> expectedX >> expectedY >> expectedGroup)
      || expectedID != id || x != expectedX || y != expectedY || group != expectedGroup)
      mismatch("construction placement differs");
  } else {
    *output << "P " << id << ' ' << x << ' ' << y << ' ' << group << '\n';
    checkOutput();
  }
  ids.insert_or_assign(building, id);
}

void SimulationTrace::visitOrder(char phase, std::uint64_t time, Order& order) {
  ++events;
  if(replaying()) {
    expect(phase);
    std::uint64_t recordedTime;
    std::size_t count;
    if(!(*input >> recordedTime >> count) || time != recordedTime || count != order.size())
      mismatch("construction visit time or count differs");
    std::unordered_map<std::uint64_t, Order::value_type> available;
    for(auto it : order)
      available.emplace(ids.at(*it), it);
    for(auto& it : order) {
      std::uint64_t id;
      if(!(*input >> id)) mismatch("missing construction visit");
      auto entry = available.find(id);
      if(entry == available.end()) mismatch("duplicate or unknown construction visit");
      it = entry->second;
      available.erase(entry);
    }
  } else {
    *output << phase << ' ' << time << ' ' << order.size();
    for(auto it : order) *output << ' ' << ids.at(*it);
    *output << '\n';
    checkOutput();
  }
}
