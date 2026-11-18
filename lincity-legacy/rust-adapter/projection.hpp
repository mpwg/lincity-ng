#pragma once
#include <memory>

class ConstructionReport;
class MapPoint;
class World;
namespace rustcity {
struct Engine;
}

/// Per-world bookkeeping. The map owns its constructions and animation frames.
class RustViewProjection {
public:
  RustViewProjection();
  ~RustViewProjection();
  RustViewProjection(const RustViewProjection &) = delete;
  RustViewProjection &operator=(const RustViewProjection &) = delete;

private:
  struct State;
  std::unique_ptr<State> state;
  friend void syncRustView(World &, const rustcity::Engine &, bool,
                           RustViewProjection &);
};

void syncRustView(World &, const rustcity::Engine &, bool simulationChanged,
                  RustViewProjection &);
bool renderRustReport(const rustcity::Engine &, MapPoint, bool production,
                      ConstructionReport &);
