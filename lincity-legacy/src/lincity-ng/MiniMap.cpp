/* ---------------------------------------------------------------------- *
 * src/lincity-ng/MiniMap.cpp
 * This file is part of Lincity-NG.
 *
 * Copyright (C) 2005      David Kamphausen <david.kamphausen@web.de>
 * Copyright (C) 2024-2026 David Bears <dbear4q@gmail.com>
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

#include "MiniMap.hpp"

#include <SDL3/SDL.h>                       // for Uint16, Uint8, SDL_BUTTON...
#include <cassert>                          // for assert
#include <fmt/format.h>                     // for format
#include <libxml++/parsers/textreader.h>    // for TextReader
#include <libxml++/ustring.h>               // for ustring
#include <cstring>                          // for strlen
#include <array>                            // for array
#include <cstddef>                          // for size_t
#include <functional>                       // for bind, _1, function, _2
#include <iostream>                         // for basic_ostream, operator<<
#include <stdexcept>                        // for runtime_error

#include "Dialog.hpp"                       // for Dialog, ASK_COAL_SURVEY
#include "Game.hpp"                         // for Game
#include "GameView.hpp"                     // for GameView
#include "MainLincity.hpp"                  // for setSimulationDelay
#include "Mps.hpp"                          // for MpsFinance, MpsMap
#include "Util.hpp"                         // for getCheckButton, getButton
#include "gui/Button.hpp"                   // for Button
#include "gui/CheckButton.hpp"              // for CheckButton
#include "gui/ComponentFactory.hpp"         // for IMPLEMENT_COMPONENT_FACTORY
#include "gui/Event.hpp"                    // for Event
#include "gui/Painter.hpp"                  // for Painter
#include "gui/Rect2D.hpp"                   // for Rect2D
#include "gui/Signal.hpp"                   // for Signal
#include "gui/SwitchComponent.hpp"          // for SwitchComponent
#include "gui/Texture.hpp"                  // for Texture
#include "lincity/MapPoint.hpp"             // for MapPoint
#include "lincity/all_buildings.hpp"        // for COAL_RESERVE_SIZE
#include "lincity/commodities.hpp"          // for Commodity, CommodityRule
#include "lincity/groups.hpp"               // for GROUP_POWER_LINE, GROUP_FIRE
#include "lincity/lin-city.hpp"             // for FLAG_IS_TRANSPORT, FLAG_C...
#include "lincity/lintypes.hpp"             // for Construction, Constructio...
#include "lincity/modules/all_modules.hpp"  // for Powerline, Transport, Fire
#include "lincity/transport.hpp"            // for TRANSPORT_QUANTA, TRANSPO...
#include "lincity/world.hpp"                // for MapTile, World, Map
#include "util/xmlutil.hpp"                 // for xmlParse, unexpectedXmlAt...
#include "util/gettextutil.hpp"

using namespace std::placeholders;


/** List of mapview buttons. The "" entries separate mapview buttons that are
 * switched
 */
const char* mapViewButtons[] = {
    "MapViewNormal", "", "MapViewFood", "", "MapViewUB40", "", "MapViewPower", "",
    "MapViewFire", "", "MapViewSport", "", "MapViewHealth", "", "MapViewTraffic", "",
    "MapViewPollution", "", "MapViewCoal", "", 0};

const char* speedButtons[] = {
    "SpeedPauseButton", "SpeedNormalButton", "SpeedFastButton",
    "SpeedFastestButton", 0 };

MiniMap* miniMapPtr = 0;

MiniMap* getMiniMap()
{
    return miniMapPtr;
}

MiniMap::MiniMap()
    : mMode(NORMAL), tilesize(2), border(0), mTexture(nullptr)
{
    assert( miniMapPtr == 0 );
    miniMapPtr = this;
}

MiniMap::~MiniMap()
{
    if(miniMapPtr == this)
        miniMapPtr = 0;
}

void
MiniMap::setGame(Game *game) {
  this->game = game;
}

Commodity
MiniMap::getStuffID()
{
    return stuff_ID;
}

void
MiniMap::toggleStuffID(int step)
{
    static const Commodity commodities[]  =
    {STUFF_FOOD,STUFF_LABOR,
    STUFF_COAL,STUFF_ORE,
    STUFF_GOODS,STUFF_STEEL,
    STUFF_WASTE,STUFF_LOVOLT,
    STUFF_HIVOLT,STUFF_WATER};

    //check if we are at the beginning or the end of commodities
    if (step == 1 && stuff_ID == STUFF_WATER)
    {
        stuff_ID = STUFF_FOOD;
    }
    else if (step != 1 && stuff_ID == STUFF_FOOD)
    {
        stuff_ID = STUFF_WATER;
    }
    else //go a step forth or back
    {
        size_t pos = 0;
        while (commodities[pos] != stuff_ID && pos < sizeof(commodities)/sizeof(stuff_ID))
        {   ++pos;}
        stuff_ID = commodities[pos+step];
    }
    setDirty();
    setMapDirty();
    mapChanged();
    updateStatusMessage();
}

const World&
MiniMap::getWorld() const {
  return game->getWorld();
}




void
MiniMap::parse(xmlpp::TextReader& reader) {
  while(reader.move_to_next_attribute()) {
    xmlpp::ustring name = reader.get_name();
    xmlpp::ustring value = reader.get_value();
    if(parseAttribute(reader));
    else if(name == "width")
      width = xmlParse<float>(value);
    else if(name == "height")
      height = xmlParse<float>(value);
    else if(name == "border")
      border = xmlParse<int>(value);
    else if(name == "tilesize")
      tilesize = xmlParse<int>(value);
    else
      unexpectedXmlAttribute(reader);
  }
  reader.move_to_element();

  if(width <= 0 || height <= 0)
    throw std::runtime_error("invalid width/height");

  mFullRefresh=true;
  alreadyAttached=false;
  inside = false;

  this->stuff_ID = STUFF_FOOD;
}

Component *MiniMap::findRoot(Component *c)
{
    while(c->getParent())
    {   return findRoot(c->getParent());}
    return c;
}

void MiniMap::attachButtons()
{
    if(alreadyAttached)
    {   return;}
    alreadyAttached=true;

    Component *root=findRoot(this);

    for(int i = 0; mapViewButtons[i] != 0; ++i)
    {
        if(strlen(mapViewButtons[i]))
        {
            CheckButton* b = getCheckButton(*root, mapViewButtons[i]);
            if(i == 0)
            {   b->check();}
            b->clicked.connect(std::bind(&MiniMap::mapViewButtonClicked, this, _1, _2));
        }
    }

    // TODO: move these non-minimap components out of here
    for(int i = 0; speedButtons[i] != 0; ++i) {
        CheckButton* b = getCheckButton(*root, speedButtons[i]);
        if(i == 1)
        {   b->check();}
        b->clicked.connect(std::bind(&MiniMap::speedButtonClicked, this, _1, _2));
    }

    Button* zoomInButton = getButton(*root, "ZoomInButton");
    zoomInButton->clicked.connect(std::bind(&MiniMap::zoomInButtonClicked, this, _1));
    Button* zoomOutButton = getButton(*root, "ZoomOutButton");
    zoomOutButton->clicked.connect(std::bind(&MiniMap::zoomOutButtonClicked, this, _1));

    CheckButton* switchMinimapButton = getCheckButton(*root, "SwitchMiniMap");
    switchMinimapButton->clicked.connect(std::bind(&MiniMap::switchButton, this, _1, _2));
    switchButtons.push_back(switchMinimapButton);

    CheckButton* switchPBarButton = getCheckButton(*root, "SwitchPBar");
    switchPBarButton->clicked.connect(std::bind(&MiniMap::switchButton, this, _1, _2));
    switchButtons.push_back(switchPBarButton);

    CheckButton* switchButton = getCheckButton(*root, "SwitchGlobalMPS");
    switchButton->clicked.connect(std::bind(&MiniMap::switchButton, this, _1, _2));
    switchButtons.push_back(switchButton);

    switchButton = getCheckButton(*root, "SwitchEconomyGraph");
    switchButton->clicked.connect(std::bind(&MiniMap::switchButton, this, _1, _2));
    switchButtons.push_back(switchButton);

    Button* scrollPageDown = getButton(*root, "ScrollPageDown");
    scrollPageDown->clicked.connect(std::bind(&MiniMap::scrollPageDownButtonClicked, this, _1));
    Button* scrollPageUp = getButton(*root, "ScrollPageUp");
    scrollPageUp->clicked.connect(std::bind(&MiniMap::scrollPageUpButtonClicked, this, _1));

}

void
MiniMap::switchButton(CheckButton* button, int mousebutton)
{
    if( !alreadyAttached )
    {   return;}
    if(mousebutton == SDL_BUTTON_RIGHT )
    {   game->showHelpWindow( "dialogs" );}
    std::string active_button_name = "Switch" + getSwitchComponent(*(findRoot(this)), "MiniMapSwitch")->getActiveComponent()->getName();
    std::string buttonName = button->getName();
    // remove "Switch" prefix
    std::string switchname = std::string(buttonName, 6, buttonName.size()-1);
    switchView(switchname);
    if(buttonName == active_button_name)
      scrollPageDown(true);
}

void
MiniMap::switchView(const std::string& viewname)
{
    SwitchComponent* switchComponent
        = getSwitchComponent(*(findRoot(this)), "MiniMapSwitch");

    switchComponent->switchComponent(viewname);
/*
    if (viewname != "EconomyGraph")
    {   getGameView()->economyGraph_open = false;}
*/
    //TODO once this gets more complex PBar pages could become
    //a nested switchComponent. For now only one case ;-)
    std::string viewGroupName = viewname;
    if (viewGroupName == "PBar2nd")
    {   viewGroupName = "PBar";}

    std::string buttonname = "Switch";
    buttonname += viewGroupName;

    for(std::vector<CheckButton*>::iterator i = switchButtons.begin();
            i != switchButtons.end(); ++i)
    {
        CheckButton* cbutton = *i;
        if(cbutton->getName() == buttonname)
        {   cbutton->check();}
        else
        {   cbutton->uncheck();}
    }
}

void
MiniMap::switchMapViewButton(const std::string &buttonName)
{
    return; //no switch atm
    std::string switchName;
    if(buttonName=="MapViewFood" || buttonName=="MapViewUB40") {
        switchName = "FoodSwitch";
    } else if(buttonName=="MapViewTraffic" || buttonName=="MapViewPollution") {
        switchName = "TrafficSwitch";
    } else if(buttonName=="MapViewPower" || buttonName=="MapViewHealth"
            || buttonName=="MapViewFire" || buttonName=="MapViewSport") {
        switchName = "PowerSwitch";
    }

    if(switchName != "") {
        SwitchComponent *switchComponent
            = getSwitchComponent(*(findRoot(this)), switchName);
        switchComponent->switchComponent(buttonName);
    }
}

MiniMap::DisplayMode getMode(const std::string &pName)
{
    if(pName=="MapViewNormal")
        return MiniMap::NORMAL;
    if(pName=="MapViewUB40")
        return MiniMap::UB40;
    if(pName=="MapViewPollution")
        return MiniMap::POLLUTION;
    if(pName=="MapViewTraffic")
        return MiniMap::TRAFFIC;
    if(pName=="MapViewCommodities")
        return MiniMap::COMMODITIES;
    if(pName=="MapViewFood")
        return MiniMap::STARVE;
    if(pName=="MapViewPower")
        return MiniMap::POWER;
    if(pName=="MapViewFire")
        return MiniMap::FIRE;
    if(pName=="MapViewSport")
        return MiniMap::CRICKET;
    if(pName=="MapViewHealth")
        return MiniMap::HEALTH;
    if(pName=="MapViewCoal")
        return MiniMap::COAL;

    return MiniMap::NORMAL;
}

std::string getNextButton(const std::string &pName)
{
    int i;
    for(i=0; mapViewButtons[i]; ++i)
        if(pName==mapViewButtons[i])
            break;
    assert(mapViewButtons[i]);

    i++;
    if(strlen(mapViewButtons[i])==0) {
        // end of row - go to beginning
        i--;
        // assuming that this is processed from front to back
        while(i>=0 && mapViewButtons[i] && strlen(mapViewButtons[i]))
        {   --i;}
        i++; // gone one too far
    }
#ifdef DEBUG
    assert(i>=0 && mapViewButtons[i] && strlen(mapViewButtons[i]));
#endif
    return mapViewButtons[i];
}

void MiniMap::mapViewChangeDisplayMode(DisplayMode newMode)
{
    Component *root = findRoot(this);
    std::string name;

    switch(newMode) {
      case NORMAL:      name="MapViewNormal"; break;
      case POLLUTION:   name="MapViewPollution"; break;
      case UB40:        name="MapViewUB40"; break;
      case STARVE:      name="MapViewFood"; break;
      case POWER:       name="MapViewPower"; break;
      case FIRE:        name="MapViewFire"; break;
      case CRICKET:     name="MapViewSport"; break;
      case HEALTH:      name="MapViewHealth"; break;
      case COAL:        name="MapViewCoal"; break;
      case TRAFFIC:     name="MapViewTraffic"; break;
      default: name="";
      //case COMMODITIES: name=//FIXME!
    }

    for(int b = 0; mapViewButtons[b] != 0; ++b)
    {
        if(strlen(mapViewButtons[b]))
        {
            CheckButton* button = getCheckButton(*root, mapViewButtons[b]);
            if(button->getName()==name)
                button->check();
            else
                button->uncheck();
        }
    }

    //FIXME there should be a way to actually use switch button
    if (newMode==mMode && mMode == TRAFFIC)
    {   newMode = COMMODITIES;}

    if(newMode == COAL) {
      if(!game->getWorld().coal_survey_done)
        new Dialog(*game, ASK_COAL_SURVEY);
    }

    mMode=newMode;
    //switchMapViewButton(name);
    switchView("MiniMap");
    setDirty();
    setMapDirty();
    mapChanged();
    updateStatusMessage();
}

void MiniMap::mapViewButtonClicked(CheckButton* button, int mousebutton)
{
    std::string name = button->getName();

    if(mousebutton == SDL_BUTTON_RIGHT ) {
    // switch button
        std::string pName=getNextButton(button->getName());

        if(pName=="MapViewNormal")
          game->showHelpWindow("msb-normal");
        else if (pName=="MapViewUB40")
          game->showHelpWindow("msb-ub40");
        else if (pName=="MapViewPollution")
          game->showHelpWindow("msb-pol");
        else if (pName=="MapViewFood")
          game->showHelpWindow("msb-starve");
        else if (pName=="MapViewPower")
          game->showHelpWindow("msb-power");
        else if (pName=="MapViewFire")
          game->showHelpWindow("msb-fire");
        else if (pName=="MapViewSport")
          game->showHelpWindow("msb-cricket");
        else if (pName=="MapViewHealth")
          game->showHelpWindow("msb-health");
        else if (pName=="MapViewCoal")
          game->showHelpWindow("msb-coal");
        else if (pName=="MapViewTraffic")
          game->showHelpWindow("msb-transport");

    return;
    }
    DisplayMode newMode=getMode(name);

    if(newMode==mMode)
    {
        // switch button
        name=getNextButton(name);
        newMode=getMode(name);
    }

    mapViewChangeDisplayMode(newMode);
}

void
MiniMap::speedButtonClicked(CheckButton* button, int)
{
    Component *root = findRoot(this);
    std::string name = button->getName();
    int i;
    for(i = 0; speedButtons[i] != 0; ++i) {
        if(name == speedButtons[i])
            break;
    }
#ifdef DEBUG
    assert(speedButtons[i] != 0);
#endif

    for(int b = 0; speedButtons[b] != 0; ++b) {
        if(b != i) {
            CheckButton* button = getCheckButton(*root, speedButtons[b]);
            button->uncheck();
        } else {
            button->check();
        }
    }

    switch(i)
    {
        case 0:
            setSimulationDelay(SIM_DELAY_PAUSE);
            break;
        case 1:
            setSimulationDelay(SIM_DELAY_SLOW);
            break;
        case 2:
            setSimulationDelay(SIM_DELAY_MED);
            break;
        case 3:
            setSimulationDelay(SIM_DELAY_FAST);
            break;
        default:
            assert(false);
            break;
    }
}

void
MiniMap::zoomInButtonClicked(Button* ) {
  game->getGameView().zoomIn();
}

void
MiniMap::zoomOutButtonClicked(Button* ) {
  game->getGameView().zoomOut();
}

void
MiniMap::scrollPageDownButtonClicked(Button* )
{
    scrollPageDown(true);
}

void
MiniMap::scrollPageUpButtonClicked(Button* )
{
    scrollPageDown(false);
}

void
MiniMap::scrollPageDown(bool down) {
  std::string viewname = getSwitchComponent(*(findRoot(this)), "MiniMapSwitch")->getActiveComponent()->getName();

  if(viewname == "MapMPS") {
    game->getMpsMap().scroll();
  }
  else if(viewname == "MiniMap") {
    toggleStuffID(down ? -1 : 1);
  }
  else if(viewname == "PBar") {
    switchView("PBar2nd");
  }
  else if(viewname == "PBar2nd") {
    switchView("PBar");
  }
  else if(viewname == "GlobalMPS") {
    // TODO: move to MpsFinance::scroll
    MpsFinance& mps = game->getMpsFinance();
    switch(mps.page) {
    case MpsFinance::Page::CASH_FLOW:
      mps.page = down
        ? MpsFinance::Page::COSTS
        : MpsFinance::Page::POPULATION;
      break;
    case MpsFinance::Page::COSTS:
      mps.page = down
        ? MpsFinance::Page::POPULATION
        : MpsFinance::Page::CASH_FLOW;
      break;
    case MpsFinance::Page::POPULATION:
      mps.page = down
        ? MpsFinance::Page::CASH_FLOW
        : MpsFinance::Page::COSTS;
      break;
    }
    mps.refresh();
  }
#ifdef DEBUG
  else if(viewname == "EconomyGraph") {
    //has only one page for now
  }
  else {
      std::cout << "Unknown active component in MiniMapSwitch: " << viewname << std::endl;
      assert(false);
  }
#endif
}


Vector2
MiniMap::mapPointToVector(MapPoint p)
{
    return Vector2((p.x - anchor.x) * tilesize , (p.y - anchor.y) * tilesize);
}

void MiniMap::constrainPosition() {
  Vector2 min(1,1);
  Vector2 max(game->getWorld().map.len()-1, game->getWorld().map.len()-1);
  max -= getSize() / tilesize;

  if(min.x > max.x) {
    anchor.x = (min.x + max.x) / 2;
  }
  else if(anchor.x < min.x) {
    anchor.x = min.x;
  }
  else if(anchor.x > max.y) {
    anchor.x = max.x;
  }

  if(min.y > max.y) {
    anchor.y = (min.y + max.y) / 2;
  }
  else if(anchor.y < min.y) {
    anchor.y = min.y;
  }
  else if(anchor.y > max.y) {
    anchor.y = max.y;
  }
}

/*
 *  Set the Corners of the GameView to show in Minimap
 */
void MiniMap::setGameViewCorners(
  Vector2 upperLeft, Vector2 lowerRight
) {
    this->upperLeft = upperLeft;
    this->lowerRight = lowerRight;
    anchor = (upperLeft + lowerRight - getSize() / tilesize) / 2;
    constrainPosition();
    setDirty();
}

void MiniMap::draw(Painter &painter) {
  attachButtons();
  if(mFullRefresh) {
    refreshTexture(painter);
    mFullRefresh = false;
  }
  painter.pushClipRect(Rect2D(Vector2(0,0), getSize()));
  int len = game->getWorld().map.len();
  Rect2D textureRect(1, 1, len - 1, len - 1);
  textureRect.move(-anchor);
  textureRect.p1 *= tilesize;
  textureRect.p2 *= tilesize;
  painter.drawStretchTexture(mTexture.get(), textureRect);
  painter.popClipRect();
}

void MiniMap::refreshTexture(Painter& painter) {
  if(!mTexture) {
    const float textureSize = game->getWorld().map.len() - 2;
    mTexture = painter.createTargetTexture(textureSize, textureSize);
    mTexture->setScaleMode(Texture::ScaleMode::NEAREST);
  }

  painter.pushRenderTarget(mTexture.get());
  painter.setFillColor(Color(0,0,0));
  painter.clear();

  bool tileMode = mMode == COAL || mMode == POLLUTION;

  for(const MapTile& tile : game->getWorld().map) {
    if(!game->getWorld().map.is_visible(tile.point))
      continue;
    if(tile.getReportingConstruction() && !tile.getConstruction() && !tileMode)
      continue;

    const int size = tileMode ? 1 : tile.getConstructionGroup()->size;
    painter.setFillColor(getColor(tile));
    painter.fillRectangle(Rect2D(
      tile.point.x - 1,
      tile.point.y - 1,
      tile.point.x - 1 + size,
      tile.point.y - 1 + size
    ));
  }

  painter.popRenderTarget();
}

Color MiniMap::getColorNormal(const MapTile& tile) const {
  auto c = MapOverlay::normal(tile);
  return Color(c.r, c.g, c.b);
}

Color MiniMap::getColor(const MapTile& tile) const {
  auto c = MapOverlay::colour(game->getWorld(), tile, mMode, stuff_ID);
  return Color(c.r, c.g, c.b);
}

void MiniMap::event(const Event& event) {

    // int left, top;
    //
    // left = (upperLeft.x + lowerRight.x) / 2 - (width / tilesize / 2);
    // top  = (upperLeft.y + lowerRight.y) / 2 - (height / tilesize / 2);

    if(event.type == Event::MOUSEMOTION) {
        if(!event.inside) {
            inside = false;
            return;
        }
        if(!inside) { //mouse just enterd the minimap, show current mapmode
            updateStatusMessage();
            inside = true;
        }
        return;
    }


    if(!event.inside) {
        return;
    }
    if(event.type==Event::MOUSEBUTTONDOWN) {
        // get Tile, that was clicked
        MapPoint tile (
          (int)((event.mousepos.x - border) / tilesize + anchor.x),
          (int)((event.mousepos.y - border) / tilesize + anchor.y)
        );

        if(event.mousebutton == SDL_BUTTON_LEFT )
        {   game->getGameView().show(tile);} // move main-map
/*
        if(event.mousebutton == SDL_BUTTON_RIGHT ) {

        }
*/
    } else if (event.type == Event::MOUSEWHEEL) {
        if (event.scrolly > 0) {
            scrollPageDown(false);
        }
        if (event.scrolly < 0) {
            scrollPageDown(true);
        }
    }
/*
    else if (event.type == Event::KEYDOWN)
    {
        //moved 'm' and 'n' for pagescrolling to Gameview
    }
*/
}

void
MiniMap::updateStatusMessage() {
  std::string m;
  switch(mMode) {
  case MiniMap::NORMAL:
    m = _("Minimap: outline map");
    break;
  case MiniMap::UB40:
    m = _("Minimap: unemployment");
    break;
  case MiniMap::POLLUTION:
    m = _("Minimap: pollution");
    break;
  case MiniMap::STARVE:
    m = _("Minimap: nourishments");
    break;
  case MiniMap::POWER:
    m = _("Minimap: power supply");
    break;
  case MiniMap::FIRE:
    m = _("Minimap: firedepartment cover");
    break;
  case MiniMap::CRICKET:
    m = _("Minimap: sport cover");
    break;
  case MiniMap::HEALTH:
    m = _("Minimap: medical care");
    break;
  case MiniMap::COAL:
    m = _("Minimap: coal deposits");
    break;
  case MiniMap::TRAFFIC:
    m = fmt::format(_("Minimap: traffic density: {}"),
      commodityNames[stuff_ID]);
    break;
  case MiniMap::COMMODITIES:
    m = fmt::format(_("Minimap: commodities: {}"), commodityNames[stuff_ID]);
    break;
  default:
    fmt::println(stderr, "error: unknown minimap mode: {}",
      static_cast<std::underlying_type_t<DisplayMode>>(mMode)
    );
    assert(false);
  }
  game->getStatusParagraph().setText(m);
}

IMPLEMENT_COMPONENT_FACTORY(MiniMap)


/** @file lincity-ng/MiniMap.cpp */
