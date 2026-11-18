// Compare bridge/crossing sprites and the corresponding moving commuter frames.
#include "lincity/Vehicles.hpp"
#include "lincity/groups.hpp"
#include "lincity/modules/track_road_rail.hpp"
#include "lincity/world.hpp"
#include <array>
#include <iostream>

namespace {
const MapPoint site(12, 12);
const std::array<MapPoint,4> directions{{{0,-1},{-1,0},{0,1},{1,0}}};

MapPoint step(MapPoint p,MapPoint d,int n) {
  return MapPoint(p.x+d.x*n,p.y+d.y*n);
}
void bridges() {
  for(auto pair : {std::pair{&trackConstructionGroup,&trackbridgeConstructionGroup},
    std::pair{&roadConstructionGroup,&roadbridgeConstructionGroup},
    std::pair{&railConstructionGroup,&railbridgeConstructionGroup}}) {
    for(const auto d : directions) {
      World w(32);
      const auto at=[&](int n){return step(site,d,n);};
      for(int n:{1,0,-1}) {
        if(n==1) w.map(at(n))->setTerrain(GROUP_WATER);
        (n==1?pair.second:pair.first)->placeItem(w,at(n));
      }
      w.map.connect_transport(1,1,30,30);
      std::cout<<"bridge "<<pair.first->group<<' '<<d.x<<' '<<d.y;
      for(int n:{1,0,-1})
        std::cout<<' '<<w.map(at(n))->getType()<<' '<<bool(w.map(at(n))->getConstruction()->flags&FLAG_TRANSPARENT);
      std::cout<<'\n';
    }
  }
}

void vehicle(std::array<MapPoint,6> path,bool bridge,bool rail) {
  World w(32);
  for(unsigned i=0;i<path.size();++i) {
    if(i==2 && bridge) w.map(path[i])->setTerrain(GROUP_WATER);
    auto& group=i==2 && rail?railConstructionGroup:roadConstructionGroup;
    group.placeItem(w,path[i]);
    w.map(path[i])->getConstruction()->commodityCount[STUFF_LABOR]=512;
  }
  // A rail yard has a larger capacity. Give its commuters the same fill ratio.
  if(rail) w.map(path[2])->getConstruction()->commodityCount[STUFF_LABOR]=2560;
  w.map.connect_transport(1,1,30,30);
  Vehicle v(w,path[0],VEHICLE_BLUECAR,VEHICLE_STRATEGY_MAXIMIZE);
  for(auto time : {1UL,52UL,239UL,615UL,1200UL,2304UL,3150UL,4600UL}) {
    v.update(time);
    std::cout<<time<<' '<<v.point.x<<' '<<v.point.y<<' '<<v.prev.x<<' '<<v.prev.y
      <<' '<<v.next.x<<' '<<v.next.y<<' '<<v.framePt.x<<' '<<v.framePt.y<<' '
      <<v.frameIt->frame<<' '<<v.frameIt->move_x<<' '<<v.frameIt->move_y<<' '<<v.direction
      <<' '<<v.speed<<' '<<v.anim<<' '<<v.death_counter<<'\n';
  }
}
}

int main(){
  bridges();
  for(auto a:directions) for(auto b:directions) {
    if(a.x*b.x+a.y*b.y!=0)continue;
    std::cout<<"turn "<<a.x<<' '<<a.y<<' '<<b.x<<' '<<b.y<<'\n';
    vehicle({step(site,a,-1),site,step(site,b,1),step(site,b,2),step(site,b,3),step(site,b,4)},false,false);
  }
  for(bool bridge:{false,true}){
    std::cout<<(bridge?"vehicle bridge":"vehicle rail")<<'\n';
    vehicle({site,site.e(),site.e(2),site.e(3),site.e(4),site.e(5)},bridge,!bridge);
  }
}
