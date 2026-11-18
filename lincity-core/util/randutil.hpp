/* ---------------------------------------------------------------------- *
 * src/util/randutil.hpp
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

#ifndef __LINCITYNG_UTIL_RANDUTIL_HPP__
#define __LINCITYNG_UTIL_RANDUTIL_HPP__

#include <initializer_list>
#include <random>

int simulationRand();

// These helpers trace the result consumed by the simulation, including the
// integer tile offset for rocket crashes. Site names are part of the tape.
bool simulationBernoulli(const char* site, double probability);
int simulationBinomial(const char* site, int trials, double probability);
int simulationUniform(const char* site, int lower, int upper);
int simulationDiscrete(const char* site, std::initializer_list<double> weights);
int simulationNormalOffset(const char* site, double deviation, int origin);

class BasicUrbg {
private:
  using base_engine = std::default_random_engine;

  BasicUrbg();
  ~BasicUrbg();
  BasicUrbg(const BasicUrbg&) = delete;
  const BasicUrbg& operator=(const BasicUrbg&) = delete;

public:
  using result_type = base_engine::result_type;

  result_type operator()();
  void seed(result_type value);
  static constexpr result_type min() { return base_engine::min(); }
  static constexpr result_type max() { return base_engine::max(); }

  static BasicUrbg& get();

private:
  base_engine base_urbg;
};

#endif // __LINCITYNG_UTIL_RANDUTIL_HPP__
