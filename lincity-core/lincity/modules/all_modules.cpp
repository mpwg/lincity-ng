/* ---------------------------------------------------------------------- *
 * src/lincity/modules/all_modules.cpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 1995-1997 I J Peters
 * Copyright (C) 1997-2005 Greg Sharp
 * Copyright (C) 2000-2004 Corey Keasling
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

#include "all_modules.hpp"

#include <map>                    // for map
#include <string>                 // for basic_string, operator<

#include "lincity/lintypes.hpp"     // for ConstructionGroup
#include "lincity/resources.hpp"  // for ResourceGroup

static void registerModules() {
    ConstructionGroup::clearGroupMap();
    ConstructionGroup::addConstructionGroup(&fireStationConstructionGroup);
    ConstructionGroup::addConstructionGroup(&healthCentreConstructionGroup);
    ConstructionGroup::addConstructionGroup(&cricketConstructionGroup);
    ConstructionGroup::addConstructionGroup(&potteryConstructionGroup);
    ConstructionGroup::addConstructionGroup(&monumentConstructionGroup);
    ConstructionGroup::addConstructionGroup(&blacksmithConstructionGroup);
    ConstructionGroup::addConstructionGroup(&waterwellConstructionGroup);
    ConstructionGroup::addConstructionGroup(&communeConstructionGroup);
    ConstructionGroup::addConstructionGroup(&windmillConstructionGroup);
    ConstructionGroup::addConstructionGroup(&windpowerConstructionGroup);
    ConstructionGroup::addConstructionGroup(&coal_powerConstructionGroup);
    ConstructionGroup::addConstructionGroup(&solarPowerConstructionGroup);
    ConstructionGroup::addConstructionGroup(&substationConstructionGroup);
    ConstructionGroup::addConstructionGroup(&millConstructionGroup);
    ConstructionGroup::addConstructionGroup(&organic_farmConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceLLConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceMLConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceHLConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceLHConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceMHConstructionGroup);
    ConstructionGroup::addConstructionGroup(&residenceHHConstructionGroup);
    ConstructionGroup::addConstructionGroup(&schoolConstructionGroup);
    ConstructionGroup::addConstructionGroup(&universityConstructionGroup);
    ConstructionGroup::addConstructionGroup(&trackConstructionGroup);
    ConstructionGroup::addConstructionGroup(&roadConstructionGroup);
    ConstructionGroup::addConstructionGroup(&railConstructionGroup);
    ConstructionGroup::addConstructionGroup(&trackbridgeConstructionGroup);
    ConstructionGroup::addConstructionGroup(&roadbridgeConstructionGroup);
    ConstructionGroup::addConstructionGroup(&railbridgeConstructionGroup);
    ConstructionGroup::addConstructionGroup(&powerlineConstructionGroup);
    ConstructionGroup::addConstructionGroup(&parklandConstructionGroup);
    ConstructionGroup::addConstructionGroup(&parkpondConstructionGroup);
    ConstructionGroup::addConstructionGroup(&oremineConstructionGroup);
    ConstructionGroup::addConstructionGroup(&coalmineConstructionGroup);
    ConstructionGroup::addConstructionGroup(&tipConstructionGroup);
    ConstructionGroup::addConstructionGroup(&recycleConstructionGroup);
    ConstructionGroup::addConstructionGroup(&portConstructionGroup);
    ConstructionGroup::addConstructionGroup(&fireConstructionGroup);
    ConstructionGroup::addConstructionGroup(&industryLightConstructionGroup);
    ConstructionGroup::addConstructionGroup(&industryHeavyConstructionGroup);
    ConstructionGroup::addConstructionGroup(&marketConstructionGroup);
    ConstructionGroup::addConstructionGroup(&rocketPadConstructionGroup);
    ConstructionGroup::addConstructionGroup(&shantyConstructionGroup);


    ConstructionGroup::clearResourcepMap();
    ConstructionGroup::addResourceID("Water", &waterConstructionGroup);
    ConstructionGroup::addResourceID("Green", &bareConstructionGroup);
    ConstructionGroup::addResourceID("Tree", &treeConstructionGroup);
    ConstructionGroup::addResourceID("Tree2", &tree2ConstructionGroup);
    ConstructionGroup::addResourceID("Tree3", &tree3ConstructionGroup);
    ConstructionGroup::addResourceID("Desert", &desertConstructionGroup);

    ConstructionGroup::addResourceID("Blacksmith", &blacksmithConstructionGroup);
    ConstructionGroup::addResourceID("FireStation", &fireStationConstructionGroup);
    ConstructionGroup::addResourceID("Health", &healthCentreConstructionGroup);
    ConstructionGroup::addResourceID("SportsCroud", &cricketConstructionGroup);
    ConstructionGroup::addResourceID("Pottery", &potteryConstructionGroup);
    //ConstructionGroup::addResourceID("Monument", &monumentFinishedConstructionGroup);
    ConstructionGroup::addResourceID("MonumentConstruction", &monumentConstructionGroup);
    ResourceGroup::registerGroup("Monument");
    ConstructionGroup::addResourceID("Waterwell", &waterwellConstructionGroup);
    ConstructionGroup::addResourceID("Commune", &communeConstructionGroup);
    ConstructionGroup::addResourceID("WindMill", &windmillConstructionGroup);
    ConstructionGroup::addResourceID("WindMillHTech", &windpowerConstructionGroup);
    ResourceGroup::registerGroup("WindMillHTechRG");
    ResourceGroup::registerGroup("WindMillHTechG");
//  ConstructionGroup::addResourceID("WindMillHTechRG", &windpower_RG_ConstructionGroup);
//  ConstructionGroup::addResourceID("WindMillHTechG", &windpower_G_ConstructionGroup);
    ConstructionGroup::addResourceID("PowerCoalEmpty", &coal_powerConstructionGroup);
    ResourceGroup::registerGroup("PowerCoalLow");
    ResourceGroup::registerGroup("PowerCoalMed");
    ResourceGroup::registerGroup("PowerCoalFull");
    //ConstructionGroup::addResourceID("PowerCoalLow", &coal_power_low_ConstructionGroup);
    //ConstructionGroup::addResourceID("PowerCoalMed", &coal_power_med_ConstructionGroup);
    //ConstructionGroup::addResourceID("PowerCoalFull", &coal_power_full_ConstructionGroup);
    ConstructionGroup::addResourceID("PowerSolar", &solarPowerConstructionGroup);
    ResourceGroup::registerGroup("Substation");
    ConstructionGroup::addResourceID("SubstationOff", &substationConstructionGroup);
    //ConstructionGroup::addResourceID("Substation", &substation_RG_ConstructionGroup);
    ResourceGroup::registerGroup("SubstationOn");
    //ConstructionGroup::addResourceID("SubstationOn", &substation_G_ConstructionGroup);
    ConstructionGroup::addResourceID("Mill", &millConstructionGroup);
    ConstructionGroup::addResourceID("Farm", &organic_farmConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialLowLow", &residenceLLConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialMedLow", &residenceMLConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialHighLow", &residenceHLConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialLow", &residenceLHConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialMedHigh", &residenceMHConstructionGroup);
    ConstructionGroup::addResourceID("ResidentialHighHigh", &residenceHHConstructionGroup);
    ConstructionGroup::addResourceID("School", &schoolConstructionGroup);
    ConstructionGroup::addResourceID("University", &universityConstructionGroup);
    ConstructionGroup::addResourceID("Track", &trackConstructionGroup);
    ConstructionGroup::addResourceID("Road", &roadConstructionGroup);
    ConstructionGroup::addResourceID("Rail", &railConstructionGroup);
    ConstructionGroup::addResourceID("TrackBridge", &trackbridgeConstructionGroup);
    ConstructionGroup::addResourceID("RoadBridge", &roadbridgeConstructionGroup);
    ConstructionGroup::addResourceID("RailBridge", &railbridgeConstructionGroup);
    ConstructionGroup::addResourceID("PowerLine", &powerlineConstructionGroup);
    ConstructionGroup::addResourceID("ParklandPlane", &parklandConstructionGroup);
    ConstructionGroup::addResourceID("ParklandLake", &parkpondConstructionGroup);
    ConstructionGroup::addResourceID("OreMine", &oremineConstructionGroup);
    ConstructionGroup::addResourceID("CoalMine", &coalmineConstructionGroup);
    ResourceGroup::registerGroup("CoalMineLow");
    ResourceGroup::registerGroup("CoalMineMed");
    ResourceGroup::registerGroup("CoalMineFull");
    //ConstructionGroup::addResourceID("CoalMineLow", &coalmine_L_ConstructionGroup);
    //ConstructionGroup::addResourceID("CoalMineMed", &coalmine_M_ConstructionGroup);
    //ConstructionGroup::addResourceID("CoalMineFull", &coalmine_H_ConstructionGroup);
    ConstructionGroup::addResourceID("Tip", &tipConstructionGroup);
    ConstructionGroup::addResourceID("Recycle", &recycleConstructionGroup);
    ConstructionGroup::addResourceID("Harbor", &portConstructionGroup);
    ConstructionGroup::addResourceID("Fire", &fireConstructionGroup);
    ResourceGroup::registerGroup("FireWasteLand");
    //ConstructionGroup::addResourceID("FireWasteLand", &fireWasteLandConstructionGroup);
    ConstructionGroup::addResourceID("IndustryLight", &industryLightConstructionGroup);
    ResourceGroup::registerGroup("IndustryLightQ");
    ResourceGroup::registerGroup("IndustryLightL");
    ResourceGroup::registerGroup("IndustryLightM");
    ResourceGroup::registerGroup("IndustryLightH");
    //ConstructionGroup::addResourceID("IndustryLightQ", &industryLight_Q_ConstructionGroup);
    //ConstructionGroup::addResourceID("IndustryLightL", &industryLight_L_ConstructionGroup);
    //ConstructionGroup::addResourceID("IndustryLightM", &industryLight_M_ConstructionGroup);
    //ConstructionGroup::addResourceID("IndustryLightH", &industryLight_H_ConstructionGroup);
    ConstructionGroup::addResourceID("IndustryHigh", &industryHeavyConstructionGroup);
    ResourceGroup::registerGroup("IndustryHighL");
    ResourceGroup::registerGroup("IndustryHighM");
    ResourceGroup::registerGroup("IndustryHighH");
    //ConstructionGroup::addResourceID("IndustryHighL", &industryHeavy_L_ConstructionGroup);
    //ConstructionGroup::addResourceID("IndustryHighM", &industryHeavy_M_ConstructionGroup);
    //ConstructionGroup::addResourceID("IndustryHighH", &industryHeavy_H_ConstructionGroup);
    ConstructionGroup::addResourceID("MarketEmpty", &marketConstructionGroup);
    ResourceGroup::registerGroup("MarketLow");
    ResourceGroup::registerGroup("MarketMed");
    ResourceGroup::registerGroup("MarketFull");
    //ConstructionGroup::addResourceID("MarketLow", &market_low_ConstructionGroup);
    //ConstructionGroup::addResourceID("MarketMed", &market_med_ConstructionGroup);
    //ConstructionGroup::addResourceID("MarketFull", &market_full_ConstructionGroup);
    ConstructionGroup::addResourceID("Rocket", &rocketPadConstructionGroup);
    ConstructionGroup::addResourceID("Shanty", &shantyConstructionGroup);

    //resources for extra sprites
    ResourceGroup::registerGroup("ChildOnSwing"); //used at School
    ResourceGroup::registerGroup("BlackSmoke");   //used at Coalpower Station
    ResourceGroup::registerGroup("GraySmoke");    //used at light Industry

    //resources for vehicles
    ResourceGroup::registerGroup("Bluecar");
    ResourceGroup::resMap["Bluecar"]->is_vehicle = true;


}

void initializeModules() {
    static const bool initialized = (registerModules(), true);
    (void)initialized;
}

std::list<const ConstructionGroup *> inventionsAfter(int techLevel) {
  std::list<const ConstructionGroup *> inventions{
    &potteryConstructionGroup,       //  0.0
    &monumentConstructionGroup,      //  0.0
    &waterwellConstructionGroup,     //  0.0
    &communeConstructionGroup,       //  0.0
    &organic_farmConstructionGroup,  //  0.0
    &trackConstructionGroup,         //  0.0
    // &residenceLLConstructionGroup,   //  0.0
    // &residenceMLConstructionGroup,   //  0.0
    &residenceHLConstructionGroup,   //  0.0
    &oremineConstructionGroup,       //  0.0
    &tipConstructionGroup,           //  0.0
    &marketConstructionGroup,        //  0.0
    &schoolConstructionGroup,        //  0.1
    &parklandConstructionGroup,      //  0.2
    // &parkpondConstructionGroup       //  0.2
    &blacksmithConstructionGroup,    //  0.3
    &cricketConstructionGroup,       //  1.2
    &fireStationConstructionGroup,   //  2.2
    &millConstructionGroup,          //  2.5
    &windmillConstructionGroup,      //  3.0
    &portConstructionGroup,          //  3.5
    &roadConstructionGroup,          //  5.0
    &coalmineConstructionGroup,      //  8.5
    &healthCentreConstructionGroup,  // 11.0
    &universityConstructionGroup,    // 15.0
    &industryLightConstructionGroup, // 16.0
    &industryHeavyConstructionGroup, // 17.0
    &railConstructionGroup,          // 18.0
    &coal_powerConstructionGroup,    // 20.0
    // &substationConstructionGroup,    // 20.0
    // &powerlineConstructionGroup,     // 20.0
    &recycleConstructionGroup,       // 23.2
    // &residenceLHConstructionGroup,   // 30.0
    // &residenceMHConstructionGroup,   // 30.0
    &residenceHHConstructionGroup,   // 30.0
    &windpowerConstructionGroup,     // 45.0
    &solarPowerConstructionGroup,    // 50.0
    &rocketPadConstructionGroup,     // 75.0
  };
  inventions.sort(
    [](const ConstructionGroup *a, const ConstructionGroup *b)
    { return a->tech < b->tech; }
  );
  while(!inventions.empty()
    && inventions.front()->tech <= techLevel
  )
    inventions.pop_front();
  return inventions;
}
