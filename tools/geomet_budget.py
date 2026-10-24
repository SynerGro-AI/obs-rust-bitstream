#!/usr/bin/env python3
"""E7 GeoMet numbers: the per-install request budget and the fixture capture commands.

Local compute only; this script makes no network request.

    python3 tools/geomet_budget.py           # print the numbers and the commands
    python3 tools/geomet_budget.py --check   # exit 1 if PLAN.md doesn't state them verbatim

The commands are typed or pasted at a Command Prompt (cmd.exe) on Windows 10/11, which
ships curl.exe. They are not for a .bat file, where cmd.exe treats % differently.
This box can't reach geo.weather.gc.ca, so the fixtures are captured on the owner's PC.
"""
import math
import pathlib
import sys

# MSC Open Data Services Usage Policy: contact MSC at 86,400 requests/day or more.
MSC_CONTACT_PER_DAY = 86_400
STEP_SECONDS = 6 * 60                 # GeoMet radar time step PT6M
STEPS_PER_DAY = 86_400 // STEP_SECONDS
REQUESTS_PER_STEP = 3                 # layer-filtered GetCapabilities, GetMap, coverage GetFeatureInfo
LEGEND_PER_DAY = 1                    # one style, cached on disk, refetched at most daily
BUDGET_PER_DAY = REQUESTS_PER_STEP * STEPS_PER_DAY + LEGEND_PER_DAY
VISIBLE_HOURS_EXAMPLE = 4

UA = "SeraphinaStudio/fixture-capture (+https://github.com/SynerGro-AI/seraphina-studio)"
BASE = "https://geo.weather.gc.ca/geomet?service=WMS&version=1.3.0"
EARTH_R = 6378137.0
EDMONTON = (-113.4938, 53.5461)       # lon, lat (GeoNames Edmonton)
ODD_POINT = (-95.0, 50.0)             # the point of the Auditor's EPSG:4326 oddity
MAP_GROUND_WIDTH_M = 400_000          # "a box about 400 km wide"
OUT_W, OUT_H = 1280, 720


def merc(lon, lat):
    return (math.radians(lon) * EARTH_R,
            math.log(math.tan(math.pi / 4 + math.radians(lat) / 2)) * EARTH_R)


def box3857(lon, lat, half_w, half_h):
    x, y = merc(lon, lat)
    return ",".join(str(round(v)) for v in (x - half_w, y - half_h, x + half_w, y + half_h))


def numbers():
    visible_steps = VISIBLE_HOURS_EXAMPLE * 3600 // STEP_SECONDS
    visible_day = REQUESTS_PER_STEP * visible_steps + LEGEND_PER_DAY
    return {
        "steps_per_day": STEPS_PER_DAY,
        "budget_per_day": BUDGET_PER_DAY,
        "installs_full_budget": MSC_CONTACT_PER_DAY // BUDGET_PER_DAY,
        "installs_getmap_only": MSC_CONTACT_PER_DAY // STEPS_PER_DAY,
        "installs_getmap_plus_one": MSC_CONTACT_PER_DAY // (2 * STEPS_PER_DAY),
        "visible_requests_4h": visible_day,
        "installs_4h": MSC_CONTACT_PER_DAY // visible_day,
    }


def commands():
    lon, lat = EDMONTON
    # Mercator stretches by 1/cos(lat); widen the projected box so it is ~400 km on the ground.
    half_w = MAP_GROUND_WIDTH_M / 2 / math.cos(math.radians(lat))
    half_h = half_w * OUT_H / OUT_W
    map_box = box3857(lon, lat, half_w, half_h)
    pt_edm = box3857(lon, lat, 505, 505)          # 101 x 101 px at 10 m/px, centre pixel 50,50
    olon, olat = ODD_POINT
    pt_odd = box3857(olon, olat, 505, 505)
    fi = "&request=GetFeatureInfo&width=101&height=101&i=50&j=50"
    reqs = [
        ("caps_precip.xml", f"{BASE}&request=GetCapabilities&layer=Radar_1km_SfcPrecipType"),
        ("caps_coverage.xml", f"{BASE}&request=GetCapabilities&layer=Radar-Coverage_SfcPrecipType"),
        ("getmap_precip_edmonton_3857.png",
         f"{BASE}&request=GetMap&layers=Radar_1km_SfcPrecipType&styles=SfcPrecipType_Dis"
         f"&crs=EPSG:3857&bbox={map_box}&width={OUT_W}&height={OUT_H}&format=image/png&transparent=true"),
        ("legend_precip_dis.png",
         f"{BASE}&request=GetLegendGraphic&sld_version=1.1.0&layer=Radar_1km_SfcPrecipType"
         f"&style=SfcPrecipType_Dis&format=image/png"),
        ("coverage_outline_3857.json",
         f"{BASE}{fi}&layers=Radar-Coverage_SfcPrecipType&query_layers=Radar-Coverage_SfcPrecipType"
         f"&info_format=application/json&crs=EPSG:3857&bbox={pt_edm}"),
        ("point_edmonton_3857.txt",
         f"{BASE}{fi}&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType"
         f"&info_format=text/plain&crs=EPSG:3857&bbox={pt_edm}"),
        ("point_odd_epsg4326.json",
         f"{BASE}{fi}&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType"
         f"&info_format=application/json&crs=EPSG:4326&bbox=49.99,-95.01,50.01,-94.99"),
        ("point_odd_crs84.json",
         f"{BASE}{fi}&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType"
         f"&info_format=application/json&crs=CRS:84&bbox=-95.01,49.99,-94.99,50.01"),
        ("point_odd_3857.json",
         f"{BASE}{fi}&layers=Radar_1km_SfcPrecipType&query_layers=Radar_1km_SfcPrecipType"
         f"&info_format=application/json&crs=EPSG:3857&bbox={pt_odd}"),
    ]
    out = []
    for name, url in reqs:
        if "%" in url:  # cmd.exe would expand it
            print(f"capture URL for {name} contains %", file=sys.stderr)
            sys.exit(1)
        stem = name.rsplit(".", 1)[0]
        out.append(f'curl.exe -sS -A "{UA}" -D {stem}.headers.txt -o {name} '
                   f'-w "%{{url_effective}}\\n" "{url}" >> requests.txt')
    return out


def main():
    n = numbers()
    cmds = commands()
    if "--check" not in sys.argv[1:]:
        for k, v in n.items():
            print(f"{k}: {v:,}")
        print()
        print("\n".join(cmds))
        return
    plan = pathlib.Path(__file__).resolve().parent.parent / "PLAN.md"
    text = plan.read_text(encoding="utf-8")
    flat = " ".join(text.split())
    phrases = [
        f"{n['budget_per_day']} requests per install per day",
        f"3 × {n['steps_per_day']} + 1",
        f"about {n['installs_full_budget']} installs",
        f"86,400 ÷ {n['budget_per_day']}",
        f"{n['installs_getmap_plus_one']} installs if",
        f"{n['installs_getmap_only']} for GetMap alone",
        f"about {n['installs_4h']} installs if radar is visible 4 hours a day",
        f"{n['visible_requests_4h']} requests",
        f"Point queries made at run time are not in the {n['budget_per_day']}-a-day budget",
        "Each one would raise it",
    ]
    bad = [p for p in phrases if p not in flat]
    bad += [c for c in cmds if c not in text]
    if bad:
        for b in bad:
            print(f"PLAN.md is missing: {b}", file=sys.stderr)
        sys.exit(1)
    print(f"PLAN.md states the GeoMet budget and all {len(cmds)} capture commands")


if __name__ == "__main__":
    main()
