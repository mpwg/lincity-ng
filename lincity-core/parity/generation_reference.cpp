// Record one generated world and its decisions for cross-implementation tests.
#include "lincity/init_game.hpp"
#include "lincity/world.hpp"
#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"

#include <array>
#include <cstdlib>
#include <fstream>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
  if(argc != 9) return 2;
  try {
    const std::array generators{new_city, new_desert_city, new_temperate_city, new_swamp_city};
    const int climate = std::stoi(argv[1]);
    const int size = std::stoi(argv[2]);
    const unsigned seed = std::stoul(argv[3]);
    city_settings settings{std::stoi(argv[4]) != 0, std::stoi(argv[5]) != 0};
    std::srand(seed);
    BasicUrbg::get().seed(seed);
    const auto generate = [&](auto& tape) {
      SimulationTrace trace(tape);
      auto world = generators.at(climate)(&settings, size);
      world->save(argv[7]);
      trace.finish();
    };
    const std::string mode(argv[8]);
    if(mode == "record") {
      std::ofstream tape(argv[6]);
      generate(tape);
    } else if(mode == "replay") {
      std::ifstream tape(argv[6]);
      generate(tape);
    } else {
      return 2;
    }
  } catch(const std::exception& ex) {
    std::cerr << ex.what() << '\n';
    return 1;
  }
}
