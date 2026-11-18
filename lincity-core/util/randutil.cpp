/* ---------------------------------------------------------------------- *
 * src/util/randutil.cpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 2025      David Bears <dbear4q@gmail.com>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
** ---------------------------------------------------------------------- */

#include "randutil.hpp"
#include "simulation_trace.hpp"

#include <cstdlib>
#include <limits>
#include <stdexcept>
#include <string>

#include <algorithm>
#include <time.h>
#include <random>
#include <vector>

BasicUrbg::BasicUrbg() {
  std::vector<std::random_device::result_type> entropy(9);
  std::generate(entropy.begin(), entropy.begin() + 8, std::random_device());
  entropy[8] = time(nullptr);
  std::seed_seq seed(entropy.begin(), entropy.end());
  base_urbg.seed(seed);
}

BasicUrbg::~BasicUrbg() = default;

BasicUrbg&
BasicUrbg::get() {
  static thread_local BasicUrbg instance;
  return instance;
}

BasicUrbg::result_type
BasicUrbg::operator()() {
  return base_urbg();
}

void BasicUrbg::seed(result_type value) {
  base_urbg.seed(value);
}

int simulationRand() {
  if(auto trace = SimulationTrace::current())
    return trace->cRandom(trace->replaying() ? 0 : std::rand());
  return std::rand();
}

namespace {
template<class Distribution>
int sample(const char* site, Distribution distribution, int lower, int upper) {
  if(auto trace = SimulationTrace::current()) {
    int value = trace->replaying() ? 0 : static_cast<int>(distribution(BasicUrbg::get()));
    return trace->sample(site, value, lower, upper);
  }
  return static_cast<int>(distribution(BasicUrbg::get()));
}
}

bool simulationBernoulli(const char* site, double probability) {
  return sample(site, std::bernoulli_distribution(probability), probability == 1, probability != 0);
}

int simulationBinomial(const char* site, int trials, double probability) {
  return sample(site, std::binomial_distribution(trials, probability),
                probability == 1 ? trials : 0, probability == 0 ? 0 : trials);
}

int simulationUniform(const char* site, int lower, int upper) {
  return sample(site, std::uniform_int_distribution(lower, upper), lower, upper);
}

int simulationDiscrete(const char* site, std::initializer_list<double> weights) {
  int index = sample(site, std::discrete_distribution(weights), 0, static_cast<int>(weights.size()) - 1);
  if(SimulationTrace::current() && weights.begin()[index] == 0)
    throw std::runtime_error(std::string("unavailable distribution outcome at ") + site);
  return index;
}

int simulationNormalOffset(const char* site, double deviation, int origin) {
  // An accepted offset must also be safe to add to its map coordinate.
  int lower = std::numeric_limits<int>::min() - std::min(origin, 0);
  int upper = std::numeric_limits<int>::max() - std::max(origin, 0);
  return sample(site, std::normal_distribution(0., deviation), lower, upper);
}
