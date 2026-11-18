// Regression cases for the LinCity-NG 2.15.0 simulation, without a frontend.
#include "commodities.hpp"
#include "stats.hpp"

#include <array>
#include <iostream>
#include <stdexcept>
#include <string_view>

#define CHECK(expression) do { if (!(expression)) throw std::runtime_error(#expression); } while (false)

void commodity_save_names() {
    constexpr std::array<std::string_view, 10> names = {
        "food", "labor", "coal", "goods", "ore", "steel", "waste", "lovolt", "hivolt", "water"
    };
    CHECK(names.size() == STUFF_COUNT);
    for (Commodity c = STUFF_INIT; c < STUFF_COUNT; ++c) {
        CHECK(commodityStandardName(c) == names.at(c));
        CHECK(commodityFromStandardName(names.at(c).data()) == c);
    }
    CHECK(commodityFromStandardName("Food") == STUFF_COUNT);
    CHECK(commodityFromStandardName("") == STUFF_COUNT);
    CHECK(std::string_view(commodityStandardName(STUFF_COUNT)) == "unknown");
}

void inventory_units() {
    Stats::Inventory<> inventory{2, 3};
    CHECK(inventory.percent() == 666);
    inventory += Stats::Inventory<>{1, 3};
    CHECK(inventory.percent() == 500);
    inventory -= Stats::Inventory<>{3, 6};
    CHECK(inventory.percent() == 0);
}

void month_boundary() {
    Stats stats;
    stats.population.population_m += 17;
    stats.population.housed_m += 12;
    stats.inventory[STUFF_FOOD].amount += 2;
    stats.inventory[STUFF_FOOD].capacity += 3;
    stats.income.goods_tax += 9;
    stats.daily();
    CHECK(stats.population.population_m.acc == 17);
    CHECK(stats.history.pop.front() == 0);
    stats.monthly();
    CHECK(stats.history.pop.front() == 17);
    CHECK(stats.history.ppool.front() == 5);
    CHECK(stats.population.population_m.acc == 0);
    CHECK(stats.history.inventory[STUFF_FOOD].front().percent() == 666);
    CHECK(stats.income.goods_tax.acc == 9);
    stats.monthly();
    CHECK(stats.history.pop.front() == 0);
    CHECK(stats.history.pop.at(1) == 17);
    CHECK(stats.history.pop.size() == 190);
    CHECK(stats.history.inventory[STUFF_FOOD].size() == 2);
}

void year_boundary() {
    Stats stats;
    stats.income.goods_tax += 18;
    stats.expenses.transport += 7;
    stats.population.unemployed_history = 100;
    stats.population.births_m += 4;
    stats.yearly();
    CHECK(stats.income.goods_tax.stat == 18);
    CHECK(stats.income.goods_tax.acc == 0);
    CHECK(stats.expenses.transport.stat == 7);
    CHECK(stats.population.unemployed_history == 99);
    CHECK(stats.population.births_m.acc == 4);
    stats.yearly();
    CHECK(stats.expenses.transport.stat == 0);
}

int main() {
    for (const auto& [name, test] : std::initializer_list<std::pair<const char*, void(*)()>>{
        {"commodity_save_names", commodity_save_names}, {"inventory_units", inventory_units},
        {"month_boundary", month_boundary}, {"year_boundary", year_boundary}
    }) {
        try {
            test();
            std::cout << "PASS " << name << '\n';
        } catch (const std::exception& e) {
            std::cerr << "FAIL " << name << ": " << e.what() << '\n';
            return 1;
        }
    }
}
