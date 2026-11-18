"""Drive the installed LinCity SDL desktop on an isolated 1024×768 X server.

Requires Python/Pillow with XCB, xvfb-run, xauth, xdotool and Tesseract with
English or German language data. Pass the application data directory produced by
the desktop build. A second executable enables bidirectional save continuation.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

from PIL import ImageChops, ImageGrab, ImageOps


MARKET = (24, 24)
SUPPLY_TRACK = (23, 24)
TRACK = (28, 25)
CITY_NAME = "SDL-regression"
BACKENDS = {"cpp": "LinCity Core (C++)", "rust": "RustCity Core"}
QUICKSAVE = "quicksave.scn.gz"
CONTINUE = "9_currentGameNG.scn.gz"


def fixture():
    root = ET.fromstring("""<lc-game ldsv-version="2140"><globals>
        <total_money>500000</total_money><tech_level>1000000</tech_level>
        <people_pool>50</people_pool></globals><map size="50">
        <Construction group="6" map-x="24" map-y="24"><food>300</food>
        <goods>1000</goods><labor>20</labor></Construction>
        <Construction group="7" map-x="23" map-y="24"><food>460</food></Construction>
        <Construction group="34" map-x="27" map-y="23"><burning_days>50</burning_days>
        <days_before_spread>150</days_before_spread></Construction>
        <Construction group="24" map-x="25" map-y="27"><labor>5000</labor>
        <tech>500000</tech></Construction>
        <Construction group="23" map-x="30" map-y="30"><steps>4999</steps>
        <tech>1000000</tech><stage>building</stage><labor>200</labor>
        <goods>10000</goods><steel>240</steel></Construction>
        <recentPoint><x>25</x><y>25</y></recentPoint></map></lc-game>""")
    ET.SubElement(root.find("globals"), "given_scene").text = CITY_NAME
    return gzip.compress(ET.tostring(root), mtime=0)


def building(save, point):
    x, y = point
    return save.find(f"map/Construction[@map-x='{x}'][@map-y='{y}']")


def day(save):
    return int(save.findtext("globals/total_time"))


def require(condition, message):
    if not condition:
        raise AssertionError(message)


class Desktop:
    def __init__(self, backend, executable, assets, directory, seed, lang):
        self.dir = directory
        self.lang = lang
        self.executable = executable
        self.backend = BACKENDS[backend]
        self.facts = {"backend": self.backend, "executable": str(executable),
                      "sha256": hashlib.sha256(executable.read_bytes()).hexdigest()}
        directory.mkdir()
        self.data = directory / "data"
        self.data.mkdir()
        (self.data / CONTINUE).write_bytes(seed)
        self.env = os.environ.copy()
        for variable, subdir in (("HOME", "home"), ("XDG_CONFIG_HOME", "config"),
                                 ("XDG_DATA_HOME", "xdg-data"), ("XDG_CACHE_HOME", "cache")):
            path = directory / subdir
            path.mkdir()
            self.env[variable] = str(path)
        self.env.update(SDL_AUDIODRIVER="dummy", SDL_VIDEODRIVER="x11", LC_ALL="C", LANGUAGE="en")
        with (directory / "desktop.log").open("w") as log:
            self.process = subprocess.Popen(
                [executable, "-s", "-w", "-m", "-S", "1024x768", "--app-data", assets,
                 "--user-data", self.data], env=self.env, stdout=log, stderr=subprocess.STDOUT)

    def __enter__(self):
        return self

    def __exit__(self, error, *_):
        if error is not None:
            self.capture("failure")
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        (self.dir / "results.json").write_text(json.dumps(self.facts, indent=2) + "\n")

    def screen(self):
        return ImageGrab.grab(xdisplay=self.env["DISPLAY"]).convert("RGB")

    def capture(self, name):
        self.screen().save(self.dir / f"{name}.png")

    def wait(self, predicate, description, timeout=15):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError(f"{self.executable} exited: {self.dir / 'desktop.log'}")
            value = predicate()
            if value:
                return value
            time.sleep(0.075)
        raise AssertionError(f"{self.dir.name}: timed out waiting for {description}")

    def input(self, *args):
        subprocess.run(["xdotool", *map(str, args)], check=True, env=self.env)

    def click(self, x, y, button=1):
        self.input("mousemove", x, y, "click", button)

    def key(self, key):
        self.input("key", key)
        time.sleep(0.06)

    def ocr(self, box):
        crop = ImageOps.grayscale(self.screen().crop(box))
        crop = crop.resize((crop.width * 2, crop.height * 2)).point(lambda v: 255 if v > 85 else 0)
        data = io.BytesIO()
        crop.save(data, format="PNG")
        return subprocess.run(["tesseract", "stdin", "stdout", "-l", self.lang, "--psm", "6"],
                              input=data.getvalue(), capture_output=True, check=True).stdout.decode()

    def text(self, name, box, expected):
        def find():
            result = self.ocr(box)
            return result if all(word.lower() in result.lower() for word in expected) else None
        self.facts[name] = self.wait(find, f"visible text {expected}")
        self.capture(name)

    def save(self, name):
        path = self.data / QUICKSAVE
        path.unlink(missing_ok=True)
        self.key("F12")

        def checkpoint():
            if not path.is_file():
                return None
            try:
                data = path.read_bytes()
                return ET.fromstring(gzip.decompress(data)), data
            except (OSError, EOFError, ET.ParseError):
                return None

        root, data = self.wait(checkpoint, f"finished GUI save {name}")
        require(root.get("ldsv-version") == "2140", "unexpected save format")
        require(root.findtext("globals/given_scene") == CITY_NAME, "wrong city")
        (self.dir / f"{name}.gz").write_bytes(data)
        return root

    def center(self):
        self.input("mousemove", 512, 380)
        self.key("KP_5")

    def begin(self, ready_event):
        self.text("main-menu", (730, 216, 943, 268), ["Continue"])
        require(f"Simulation backend: {self.backend}" in (self.dir / "desktop.log").read_text(),
                f"expected {self.backend} startup marker in {self.dir / 'desktop.log'}")
        self.click(810, 242)
        if ready_event:
            self.text("rocket-ready", (252, 316, 859, 435), ["Rocket", "Launch"])
            self.click(340, 454)  # Defer launch in the actual event dialog.
        else:
            self.wait(lambda: "loaded game from" in (self.dir / "desktop.log").read_text(),
                      "desktop to load the saved city")
            time.sleep(0.25)
        self.click(24, 670)  # Pause.
        self.center()

    def inspect_market(self):
        self.click(512, 320, 3)
        self.text("market-report", (817, 526, 1013, 704), ["Market", "busy"])

    def toggle_food(self):
        self.click(512, 320)
        self.text("market-trade", (369, 190, 651, 562), ["Buy", "Sell", "Food"])
        self.click(538, 290)  # Buy Food.
        self.click(585, 580)  # Apply the checkbox states.

    def remove_track(self):
        self.key("b")
        self.click(704, 480)
        self.key("Escape")
        require(building(self.save("after-demolition"), TRACK) is None,
                f"bulldozer did not remove track {TRACK}")

    def exercise(self):
        self.begin(True)
        first = self.save("paused")
        # Only the market and its neighboring track hold food in this fixture.
        market_food = int(building(first, MARKET).findtext("food"))
        track_food = int(building(first, SUPPLY_TRACK).findtext("food"))
        require(market_food > 300 and track_food < 460 and market_food + track_food == 760,
                f"market/track food transfer differs: {market_food} / {track_food}")
        self.facts["food_after_trade"] = {"market": market_food, "supply_track": track_food}
        time.sleep(0.45)
        require(day(self.save("still-paused")) == day(first), "paused city advanced")
        self.click(56, 697)  # Normal speed.
        time.sleep(0.65)
        self.click(24, 670)
        normal = day(self.save("normal-speed"))
        require(normal > day(first), "normal speed did not advance")
        self.click(96, 715)  # Fast speed.
        time.sleep(0.65)
        self.click(24, 670)
        fast = day(self.save("fast-speed"))
        require(fast - normal > normal - day(first), "fast rate did not exceed normal rate")
        self.facts["days"] = [day(first), normal, fast]
        self.center()
        self.click(56, 697)
        self.input("mousemove", 735, 465)
        # This crop covers the seeded fire at (27,23), excluding the HUD and pointer.
        fire_box = (735, 301, 806, 410)
        original = self.screen().crop(fire_box)
        original.save(self.dir / "fire-before.png")

        def fire_change():
            current = self.screen().crop(fire_box)
            difference = ImageChops.difference(original, current).convert("L")
            changed = sum(difference.histogram()[24:])
            if changed < 100:
                return None
            current.save(self.dir / "fire-after.png")
            return changed

        self.facts["animated_fire_pixels"] = self.wait(fire_change, "fire sprite to animate", 5)
        self.click(24, 670)
        self.inspect_market()
        self.toggle_food()
        trade = self.save("trade")
        require(building(trade, MARKET).findtext("take_food") == "0", "market still buys food")
        self.key("grave")
        self.capture("normal-minimap")
        mini_box = (865, 575, 963, 672)
        terrain = self.screen().crop(mini_box)
        self.key("8")
        self.wait(lambda: ImageChops.difference(terrain, self.screen().crop(mini_box)).getbbox(),
                  "pollution overlay on minimap")
        self.capture("pollution-minimap")
        self.key("grave")
        self.click(24, 211)  # Transport menu.
        self.click(74, 211)  # Track.
        self.click(704, 480)
        built = self.save("built")
        track = building(built, TRACK)
        require(track is not None and track.get("group") == "7", f"missing built track at {TRACK}")
        self.capture("track-built")
        self.remove_track()
        # Exercise quickload in the running process, then verify a fresh GUI save.
        checkpoint = (self.dir / "built.gz").read_bytes()
        (self.data / QUICKSAVE).write_bytes(checkpoint)
        self.key("F9")
        self.wait(lambda: any("loaded game from" in line and QUICKSAVE in line
                              for line in (self.dir / "desktop.log").read_text().splitlines()),
                  "GUI quickload")
        restored = self.save("quickloaded")
        require(building(restored, TRACK) is not None, "quickload did not restore track")
        require(building(restored, MARKET).findtext("take_food") == "0", "quickload lost trade rule")
        return checkpoint

    def continue_from_other(self, before):
        self.begin(False)
        loaded = self.save("loaded")
        track = building(loaded, TRACK)
        require(track is not None and track.get("group") == "7", "cross-load lost track")
        require(building(loaded, MARKET).findtext("take_food") == "0", "cross-load lost trade rule")
        self.inspect_market()
        self.toggle_food()
        self.remove_track()
        self.click(56, 697)
        time.sleep(0.65)
        self.click(24, 670)
        result = self.save("continued")
        require(building(result, MARKET).findtext("take_food") == "1", "cross-city trade did not apply")
        require(building(result, TRACK) is None, "cross-city demolition did not persist")
        require(day(result) > day(before), "cross-city simulation did not continue")
        self.facts["continued_days"] = [day(before), day(result)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cpp", required=True, type=Path, help="C++ SDL executable")
    parser.add_argument("--rust", type=Path, help="Rust-backed SDL executable")
    parser.add_argument("--assets", required=True, type=Path, help="installed lincity-ng data directory")
    parser.add_argument("--artifacts", required=True, type=Path, help="new directory for screenshots and saves")
    parser.add_argument("--tools-prefix", type=Path, help="prefix with bin/Xvfb and bin/xdotool")
    parser.add_argument("--isolated-display", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if not args.isolated_display:
        env = os.environ.copy()
        if args.tools_prefix:
            prefix = args.tools_prefix.resolve()
            env["PATH"] = str(prefix / "bin") + os.pathsep + env["PATH"]
            env["LD_LIBRARY_PATH"] = os.pathsep.join(filter(None, [str(prefix / "lib"), env.get("LD_LIBRARY_PATH")]))
        return subprocess.run(["xvfb-run", "-a", "-s", "-screen 0 1024x768x24 -nolisten tcp",
                               sys.executable, __file__, *sys.argv[1:], "--isolated-display"], env=env).returncode
    languages = subprocess.check_output(["tesseract", "--list-langs"], text=True).splitlines()
    lang = next((x for x in ("eng", "deu") if x in languages), None)
    require(lang, "install English or German Tesseract language data")
    root = args.artifacts.resolve()
    root.mkdir(parents=True, exist_ok=False)
    assets = args.assets.resolve()
    executables = {"cpp": args.cpp.resolve()}
    if args.rust:
        executables["rust"] = args.rust.resolve()
    checkpoints = {}
    for name, exe in executables.items():
        with Desktop(name, exe, assets, root / name, fixture(), lang) as desktop:
            checkpoints[name] = desktop.exercise()
        print(f"{name}: SDL actions, speeds, reports, minimap, event, animation and quickload passed", flush=True)
    if args.rust:
        for source, target in (("cpp", "rust"), ("rust", "cpp")):
            data = checkpoints[source]
            with Desktop(target, executables[target], assets, root / f"{source}-to-{target}", data, lang) as desktop:
                desktop.continue_from_other(ET.fromstring(gzip.decompress(data)))
            print(f"{source} to {target}: cross-loaded and continued through SDL", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
