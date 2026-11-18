/* ---------------------------------------------------------------------- *
 * src/lincity-ng/Mps.cpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 2005      David Kamphausen <david.kamphausen@web.de>
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

#include "Mps.hpp"

#include <assert.h>                         // for assert
#include <libxml++/parsers/textreader.h>    // for TextReader
#include <libxml++/ustring.h>               // for ustring
#include <algorithm>                        // for max
#include <array>                            // for array
#include <deque>                            // for deque, operator!=
#include <iomanip>                          // for setprecision, _Setprecision
#include <iterator>                         // for advance
#include <memory>                           // for unique_ptr
#include <numeric>                          // for accumulate
#include <sstream>                          // for basic_ostringstream, oper...
#include <utility>                          // for move

#include "Game.hpp"                         // for Game
#include "Util.hpp"                         // for getParagraph
#include "gui/ComponentFactory.hpp"         // for IMPLEMENT_COMPONENT_FACTORY
#include "gui/ComponentLoader.hpp"          // for parseEmbeddedComponent
#include "gui/Paragraph.hpp"                // for Paragraph
#include "lincity/MapPoint.hpp"             // for MapPoint
#include "lincity/commodities.hpp"          // for Commodity
#include "lincity/groups.hpp"               // for GROUP_DESERT, GROUP_SHANTY
#include "lincity/lin-city.hpp"             // for FLAG_IS_LAKE, FLAG_IS_RIVER
#include "lincity/lintypes.hpp"             // for NUMOF_DAYS_IN_MONTH, Cons...
#include "lincity/modules/all_modules.hpp"  // for TileConstructionGroup
#include "lincity/stats.hpp"                // for Stat, Stats
#include "DisplayFormat.hpp"                 // for num_to_ansi, current_year
#include "lincity/world.hpp"                // for MapTile, World, Ground
#include "util/gettextutil.hpp"             // for N_, _
#include "util/xmlutil.hpp"                 // for unexpectedXmlAttribute

Mps::Mps() { }

Mps::~Mps() { }

void
Mps::parse(xmlpp::TextReader& reader) {
  while(reader.move_to_next_attribute()) {
    xmlpp::ustring name = reader.get_name();
    xmlpp::ustring value = reader.get_value();
    if(parseAttribute(reader));
    else
      unexpectedXmlAttribute(reader);
  }
  reader.move_to_element();

  std::unique_ptr<Component> component = parseEmbeddedComponent(reader);
  width = component->getWidth();
  height = component->getHeight();
  addChild(std::move(component));

  for(int i = 0; i < PARAGRAPH_COUNT; ++i) {
    std::ostringstream compname;
    compname << "mps_text" << i;
    Paragraph* p = getParagraph(*this, compname.str());
    paragraphs.push_back(p);
  }
}

void
Mps::reset() {
  i = 0;
}

void
Mps::clear() {
  for(auto it = paragraphs.begin(); it != paragraphs.end(); ++it)
    (*it)->setText("");
  reset();
}

void
Mps::addText(const std::string &s) {
  assert(i >= 0 && i < PARAGRAPH_COUNT);
  paragraphs[i++]->setText(s);
}

void
Mps::addBlank() {
  addText("");
}

void
Mps::fillBlank() {
  while(!isFull())
    addBlank();
}

bool
Mps::isFull() const {
  return i >= PARAGRAPH_COUNT;
}

void
Mps::addLabel(const std::string &t) {
  addText(_(t.c_str()));
}

void
Mps::add_fp(int i, double f) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << f << "%"
  ).str());
}

void
Mps::add_f(double f) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << f
  ).str());
}

void
Mps::addDecimal(const std::string& s, double fl) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s.c_str()) << "\t" << fl
  ).str());
}


void
Mps::add_d(int d) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << d
  ).str());
}

void
Mps::addLabeledText(const std::string& s1, const std::string& s2) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s1.c_str()) << "\t" << _(s2.c_str())
  ).str());
}

void
Mps::addInteger(const std::string& s, int d) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s.c_str()) << "\t" << d
  ).str());
}

void
Mps::add_ssd(const std::string& s1, const std::string& s2, int d) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s1.c_str()) << "\t" << _(s2.c_str()) << "\t" << d
  ).str());
}

void
Mps::addPercentage(const std::string& s, double fl) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s.c_str()) << "\t" << fl << "%"
  ).str());
}

void
Mps::add_sdd(const std::string& s, int d1, int d2) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s.c_str()) << "\t" << d1 << "\t" << d2
  ).str());
}

void
Mps::addQuantity(const std::string& s, int d, int max) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << _(s.c_str()) << "\t" << d << "\t" << (max ? d*100.0/max : 0.0) << "%"
  ).str());
}

void
Mps::addPrefixedQuantity(const std::string& t1, const std::string& s2, int d, int max) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << t1 << _(s2.c_str()) << "\t" << d << "\t"
    << (max ? d*100.0/max : 0.0) << "%"
  ).str());
}

void
Mps::add_ttt(const std::string& t1, const std::string& t2, const std::string& t3
) {
  addText((std::ostringstream() << std::setprecision(1) << std::fixed
    << t1 << "\t" << t2 << "\t" << t3
  ).str());
}


void
MpsMap::refresh() {
  clear();
  if(!tile) return;
  switch(page) {
  case Page::INVENTORY:
    refreshInvProd(false);
    break;
  case Page::PRODUCTION:
    refreshInvProd(true);
    break;
  case Page::GROUND:
    refreshGround();
    break;
  }
  fillBlank();
}

void
MpsMap::setTile(const MapTile *tile) {
  this->tile = tile;
  refresh();
}

void
MpsMap::scroll() {
  if(!tile) return;
  switch(page) {
  case MpsMap::Page::INVENTORY:
    page = MpsMap::Page::PRODUCTION;
    break;
  case MpsMap::Page::PRODUCTION:
    page = MpsMap::Page::INVENTORY;
    break;
  case MpsMap::Page::GROUND:
    page = MpsMap::Page::GROUND;
    break;
  default:
    assert(false);
  }
  refresh();
}

void
MpsMap::query(const MapTile *tile) {
  if(tile == this->tile) {
    scroll();
  }
  else {
    setTile(tile);
  }
}

void
MpsMap::refreshInvProd(bool production) {
  const ConstructionGroup *tileCstGrp = tile->getTileConstructionGroup();
  if(tile->reportingConstruction) {
    tile->reportingConstruction->report(*this, production);
  }
  else if(tileCstGrp == &waterConstructionGroup) {
    add_sdd(waterConstructionGroup.name, tile->point.x, tile->point.y);

    addBlank();
    const char *p;
    if(tile->flags & FLAG_IS_LAKE)
      p = N_("Lake");
    else if(tile->flags & FLAG_IS_RIVER)
      p = N_("River");
    else
      p = N_("Pond");
    addLabel(p);
  }
  else {
    add_sdd(tileCstGrp->name, tile->point.x, tile->point.y);
    addBlank();
    addLabel(N_("no further information available"));

    if(tile->is_bare()) {
      addBlank();
      addBlank();
      addBlank();
      addBlank();
      addLabel(N_("build something here"));
    }
  }
}

void
MpsMap::refreshGround() {
  const char* p;

  add_sdd(tile->getTileConstructionGroup()->name, tile->point.x, tile->point.y);
  addLabeledText(N_("Fertile"),
    (tile->flags & FLAG_HAS_UNDERGROUND_WATER) ? N_("Yes") : N_("No"));

  if(tile->group == GROUP_WATER) {
    if(tile->flags & FLAG_IS_LAKE)
      p = N_("Lake");
    else if(tile->flags & FLAG_IS_RIVER)
      p = N_("River");
    else
      p = N_("Pond");
    addLabel(p);
  }
  else {
    addBlank();
  }

  addLabeledText(N_("Fire Protection"),
    (tile->flags & FLAG_FIRE_COVER) ? N_("Yes") : N_("No"));

  addLabeledText(N_("Health Care"),
    (tile->flags & FLAG_HEALTH_COVER) ? N_("Yes") : N_("No"));

  addLabeledText(N_("Public Sports"),
    (tile->flags & FLAG_CRICKET_COVER) ? N_("Yes") : N_("No"));

  addLabeledText(N_("Market Range"),
    (tile->flags & FLAG_MARKET_COVER) ? N_("Yes") : N_("No"));

  int pol = tile->pollution;
  if(pol < 10)
    p = N_("clear");
  else if(pol < 25)
    p = N_("good");
  else if(pol < 70)
    p = N_("fair");
  else if(pol < 190)
    p = N_("smelly");
  else if(pol < 450)
    p = N_("smokey");
  else if(pol < 1000)
    p = N_("smoggy");
  else if(pol < 1700)
    p = N_("bad");
  else if(pol < 3000)
    p = N_("very bad");
  else
    p = N_("death!");

  add_ssd(N_("Air Pollution"), p, pol);

  if(tile->getGroup() == GROUP_DESERT)
    addLabeledText(N_("Bull. Cost"), N_("N/A"));
  else
    addInteger(N_("Bull. Cost"), tile->getConstructionGroup()->bul_cost);

  addInteger(N_("Ore Reserve"), tile->ore_reserve);
  addInteger(N_("Coal Reserve"), tile->coal_reserve);
  addInteger(N_("ground level"), tile->ground.altitude);

  // TODO: the Map is unreachable from here
  //       MapTile should hold a reference to the container Map
  //       MapTile should have methods getMap, isSaddle, isMinimum, etc.
#if 0
  const Map& map = tile->getMap();
  if(map.saddlepoint(point))
    p = N_("saddle point");
  else if(!tile.is_water() && map.minimum(point))
    p = N_("minimum");
  else if(!tile.is_water() && map.maximum(point))
    p = N_("maximum");
  else if(map.checkEdgeMin(point))
    p = N_("lowest edge");
  else
    p = "-";
  addLabel(p);
#endif
}


void
MpsFinance::refresh() {
  clear();
  switch(page) {
  case Page::CASH_FLOW:
    refreshCashFlow();
    break;
  case Page::COSTS:
    refreshCosts();
    break;
  case Page::POPULATION:
    refreshPopulation();
    break;
  }
  fillBlank();
}

void
MpsFinance::refreshCashFlow() {
  const Stats& stats = game->getWorld().stats;
  int amt;
  int cashflow = 0;

  addLabel(N_("Tax Income"));

  amt = stats.income.income_tax;
  addLabeledText(N_("Income"), num_to_ansi(-amt));
  cashflow -= amt;

  amt = stats.income.coal_tax;
  addLabeledText(Construction::getStuffName(STUFF_COAL), num_to_ansi(-amt));
  cashflow -= amt;

  amt = stats.income.goods_tax;
  addLabeledText(Construction::getStuffName(STUFF_GOODS), num_to_ansi(-amt));
  cashflow -= amt;

  amt = stats.income.export_tax;
  addLabeledText(N_("Export"), num_to_ansi(-amt));
  cashflow -= amt;

  addBlank();

  addLabel(N_("Expenses"));

  amt = stats.expenses.unemployment;
  addLabeledText(N_("Unemp."), num_to_ansi(amt));
  cashflow -= amt;

  amt = stats.expenses.transport;
  addLabeledText(N_("Transport"), num_to_ansi(amt));
  cashflow -= amt;

  amt = stats.expenses.import;
  addLabeledText(N_("Imports"), num_to_ansi(amt));
  cashflow -= amt;

  amt = stats.expenses.windmill
    + stats.expenses.university
    + stats.expenses.recycle
    + stats.expenses.deaths
    + stats.expenses.health
    + stats.expenses.rockets
    + stats.expenses.school
    + stats.expenses.firestation
    + stats.expenses.cricket
    + stats.expenses.interest;
  addLabeledText(N_("Others"), num_to_ansi(amt));
  cashflow -= amt;

  addBlank();

  addLabeledText(N_("Net"), num_to_ansi(cashflow));
}

void
MpsFinance::refreshCosts() {
  const Stats& stats = game->getWorld().stats;
  addLabel(N_("Other Costs"));
  addInteger(N_("For year"), current_year(game->getWorld().total_time) - 1);
  addBlank();
  addLabeledText(N_("Interest"), num_to_ansi(stats.expenses.interest));
  addLabeledText(N_("Schools") , num_to_ansi(stats.expenses.school));
  addLabeledText(N_("Univers."), num_to_ansi(stats.expenses.university));
  addLabeledText(N_("Deaths")  , num_to_ansi(stats.expenses.deaths));
  addLabeledText(N_("Windmill"), num_to_ansi(stats.expenses.windmill));
  addLabeledText(N_("Hospital"), num_to_ansi(stats.expenses.health));
  addLabeledText(N_("Rockets") , num_to_ansi(stats.expenses.rockets));
  addLabeledText(N_("Fire Stn"), num_to_ansi(stats.expenses.firestation));
  addLabeledText(N_("Sport")   , num_to_ansi(stats.expenses.cricket));
  addLabeledText(N_("Recycle") , num_to_ansi(stats.expenses.recycle));
}

void
MpsFinance::refreshPopulation() {
  const Stats& stats = game->getWorld().stats;
  addLabel(N_("Population") );
  addBlank();
  addInteger(N_("Total"), stats.population.population_m / NUMOF_DAYS_IN_MONTH);
  addInteger(N_("Housed"), stats.population.housed_m / NUMOF_DAYS_IN_MONTH);
  addInteger(N_("Homeless"),
    (stats.population.population_m - stats.population.housed_m)
    / NUMOF_DAYS_IN_MONTH);
  addInteger(N_("Shanties"), stats.groupCount[GROUP_SHANTY]);

  addQuantity(N_("Unemployment"),
    stats.population.unemployed_m / NUMOF_DAYS_IN_MONTH,
    stats.population.housed_m / NUMOF_DAYS_IN_MONTH);
  addQuantity(N_("Starvation"),
    stats.population.starving_m / NUMOF_DAYS_IN_MONTH,
    stats.population.housed_m / NUMOF_DAYS_IN_MONTH);

  assert(stats.history.births.size() >= 12);
  auto birthsEnd = stats.history.births.begin();
  std::advance(birthsEnd, 12);
  addQuantity(N_("Births p.a."),
    std::accumulate(stats.history.births.begin(), birthsEnd, 0),
    stats.population.population_m / NUMOF_DAYS_IN_MONTH);

  assert(stats.history.deaths.size() >= 12);
  auto deathsEnd = stats.history.deaths.begin();
  std::advance(deathsEnd, 12);
  int tdeaths = std::accumulate(stats.history.deaths.begin(), deathsEnd, 0);
  addQuantity(N_("Deaths p.a."), tdeaths,
    stats.population.population_m / NUMOF_DAYS_IN_MONTH);

  assert(stats.history.unnat_deaths.size() >= 12);
  auto unnatDeathsEnd = stats.history.unnat_deaths.begin();
  std::advance(unnatDeathsEnd, 12);
  addQuantity(N_("Unnat. Deaths"),
    std::accumulate(stats.history.unnat_deaths.begin(), unnatDeathsEnd, 0),
    std::max({tdeaths, 1}));
}

IMPLEMENT_COMPONENT_FACTORY(Mps)
IMPLEMENT_COMPONENT_FACTORY(MpsMap)
IMPLEMENT_COMPONENT_FACTORY(MpsFinance)


/** @file lincity-ng/Mps.cpp */
