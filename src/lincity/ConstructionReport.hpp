/* ---------------------------------------------------------------------- *
 * src/lincity/ConstructionReport.hpp
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

#ifndef LINCITY_CONSTRUCTION_REPORT_HPP
#define LINCITY_CONSTRUCTION_REPORT_HPP

#include <string>

/// Receives the ordered rows of a building's inventory or production report.
class ConstructionReport {
public:
  virtual ~ConstructionReport() = default;

  virtual void addBlank() = 0;
  virtual void addLabel(const std::string& label) = 0;
  virtual void addLabeledText(const std::string& label, const std::string& value) = 0;
  virtual void addInteger(const std::string& label, int value) = 0;
  virtual void addDecimal(const std::string& label, double value) = 0;
  virtual void addPercentage(const std::string& label, double percentage) = 0;
  virtual void addQuantity(const std::string& label, int value, int capacity) = 0;
  virtual void addPrefixedQuantity(const std::string& prefix,
    const std::string& label, int value, int capacity) = 0;
};

#endif
