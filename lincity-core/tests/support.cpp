#include "util/randutil.hpp"
#include "util/simulation_trace.hpp"
#include "util/xmlutil.hpp"

#include <libxml++/parsers/textreader.h>

#include <filesystem>
#include <iostream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <system_error>

#define CHECK(test) do { if(!(test)) throw std::runtime_error(#test); } while(false)

namespace {
constexpr const char* signature = "LINCITY-SIMULATION-TRACE 1\n";

template<class Error, class Operation>
void reject(const char* diagnostic, Operation run) {
  try {
    run();
  } catch(const Error& error) {
    CHECK(std::string(error.what()).find(diagnostic) != std::string::npos);
    return;
  }
  throw std::runtime_error(std::string("accepted invalid operation: ") + diagnostic);
}

void traceLifetime() {
  {
    std::ostringstream output;
    SimulationTrace recording(output);
    reject<std::logic_error>("already attached", [&] { SimulationTrace nested(output); });
    std::istringstream input(std::string(signature) + "Z\n");
    reject<std::logic_error>("already attached", [&] { SimulationTrace nested(input); });
    recording.finish();
    CHECK(output.str() == std::string(signature) + "Z\n");
    reject<std::logic_error>("inactive", [&] { recording.finish(); });
  }
  std::ostringstream output;
  output.setstate(std::ios::badbit);
  reject<std::runtime_error>("write failed", [&] { SimulationTrace broken(output); });
  for(const auto* header : {"", "LINCITY-SIMULATION-TRACE 2\n"}) {
    std::istringstream input(header);
    reject<std::runtime_error>("invalid header", [&] { SimulationTrace broken(input); });
  }
  CHECK(SimulationTrace::current() == nullptr);
}

template<class Operation>
void rejectReplay(const char* records, const char* reason, Operation run) {
  std::istringstream input(std::string(signature) + records);
  {
    SimulationTrace replay(input);
    reject<std::runtime_error>(reason, [&] { run(replay); });
  }
  CHECK(SimulationTrace::current() == nullptr);
}

void traceInputValidation() {
  rejectReplay("C -1\n", "outside generator range", [](auto&) { simulationRand(); });
  rejectReplay("C x\n", "outside generator range", [](auto&) { simulationRand(); });
  rejectReplay("S other 1\n", "unexpected distribution site",
               [](auto&) { simulationUniform("test", 0, 1); });
  rejectReplay("S test 2\n", "distribution outcome out of range",
               [](auto&) { simulationUniform("test", 0, 1); });
  rejectReplay("S test 0\n", "unavailable distribution outcome",
               [](auto&) { simulationDiscrete("test", {0, 1}); });
  rejectReplay("P 1 6 5 7\n", "construction placement differs",
               [](auto& replay) { replay.placed(nullptr, 5, 5, 7); });
  rejectReplay("A 0 1\n", "construction visit time or count differs", [](auto& replay) {
    SimulationTrace::Order partial;
    replay.visitOrder('A', 0, partial);
  });
  rejectReplay("P 1 5 5 7\nD 5 1\n", "missing construction visit", [](auto& replay) {
    replay.placed(nullptr, 5, 5, 7);
    std::set<Construction*> buildings{nullptr};
    SimulationTrace::Order incomplete{buildings.begin()};
    replay.visitOrder('D', 5, incomplete);
  });
}

template<class Number>
void xmlNumbers(Number value) {
  CHECK(xmlParse<Number>(xmlFormat(value)) == value);
  CHECK(xmlParse<Number>(xmlFormatHex(value)) == value);
}

void xmlTypes() {
  xmlNumbers(std::numeric_limits<int>::min());
  xmlNumbers(std::numeric_limits<unsigned short>::max());
  xmlNumbers(std::numeric_limits<std::size_t>::max());
  xmlNumbers(-0.125f);
  CHECK(xmlParse<int>("+0X12") == 18);
  CHECK(xmlParse<int>("-0x12") == -18);
  reject<std::system_error>("extra characters after XML number", [] { xmlParse<int>("12 turns"); });
  reject<std::system_error>("extra characters after XML number", [] { xmlParse<float>("1.5%"); });
  reject<std::system_error>("failed to parse XML value", [] { xmlParse<int>(""); });
  CHECK(xmlParse<bool>(xmlFormat(true)));
  CHECK(!xmlParse<bool>(xmlFormat(false)));
  CHECK(xmlParse<bool>("YES") && xmlParse<bool>("1") && !xmlParse<bool>("0"));
  const std::filesystem::path path("opening/Green City.sav.gz");
  CHECK(xmlParse<std::filesystem::path>(xmlFormat(path)) == path);
}

void xmlAttributeDiagnostics() {
  const unsigned char input[] = "<tile obsolete='west' altitude='14'/>";
  xmlpp::TextReader reader(input, sizeof(input) - 1);
  CHECK(reader.read() && reader.get_name() == "tile");
  CHECK(reader.move_to_attribute("obsolete"));
  reject<std::runtime_error>("missing XML attribute \"group\" in <tile>", [&] {
    missingXmlAttribute(reader, "group");
  });
  CHECK(reader.get_name() == "tile");
  CHECK(reader.get_attribute("altitude") == "14");
}

void warnAboutXml(std::string_view kind) {
  const unsigned char input[] = "<tile obsolete='west' altitude='14'/>";
  xmlpp::TextReader reader(input, sizeof(input) - 1);
  CHECK(reader.read() && reader.get_name() == "tile");
#ifdef NDEBUG
  std::cout << "warnings" << std::endl;
#else
  std::cout << "assertions" << std::endl;
#endif
  if(kind == "attribute") {
    CHECK(reader.move_to_attribute("obsolete"));
    unexpectedXmlAttribute(reader);
  } else {
    unexpectedXmlElement(reader);
  }
  CHECK(reader.get_name() == "tile");
  CHECK(reader.get_attribute("altitude") == "14");
}
}

int main(int argc, char** argv) {
  try {
    if(argc == 2 && (std::string_view(argv[1]) == "attribute" ||
                     std::string_view(argv[1]) == "element")) {
      warnAboutXml(argv[1]);
      return 0;
    }
    if(argc != 1) return 2;
    traceLifetime();
    traceInputValidation();
    xmlTypes();
    xmlAttributeDiagnostics();
  } catch(const std::exception& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}
