"""Measures how long the main views take with a large tree (browser app, WASM).
python3 perf.py http://localhost:4173/ big.ged [--import]"""
import os, sys, time, json
from playwright.sync_api import sync_playwright

url, ged = sys.argv[1], sys.argv[2]
VIEWS = ["/", "/people", "/tree", "/map", "/events", "/graves", "/places", "/stats", "/health", "/duplicates", "/sources", "/people", "/tree"]

SETTLE = """async () => {
  const idle = () => new Promise(r => requestIdleCallback(() => r(), {timeout: 60000}));
  const raf = () => new Promise(r => requestAnimationFrame(() => r()));
  for (let i = 0; i < 3; i++) { await raf(); await idle(); }
  return true;
}"""

with sync_playwright() as p:
    b = p.chromium.launch(executable_path=os.environ.get("CHROME", "/usr/bin/google-chrome"), args=["--no-sandbox"])
    ctx = b.new_context(viewport={"width": 1440, "height": 900}, locale="pl-PL")
    page = ctx.new_page()
    errors = []
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.on("console", lambda m: m.type == "error" and errors.append(m.text))
    page.add_init_script("""
      window.__lt = 0; window.__ltMax = 0;
      new PerformanceObserver(l => { for (const e of l.getEntries()) { window.__lt += e.duration; window.__ltMax = Math.max(window.__ltMax, e.duration); } }).observe({type: 'longtask', buffered: true});
    """)
    page.goto(url + "#/data?tab=gedcom")
    page.wait_for_timeout(1500)
    page.evaluate(SETTLE)
    t = time.time()
    with page.expect_file_chooser() as fc:
        page.get_by_role("button", name="Wybierz plik .ged").click()
    fc.value.set_files(ged)
    page.get_by_role("button", name="Import", exact=True).wait_for(timeout=180000)
    print(f"dry run: {time.time()-t:.1f}s")
    t = time.time()
    page.get_by_role("button", name="Import", exact=True).click()
    page.get_by_text("Import zakończony").wait_for(timeout=600000)
    page.evaluate(SETTLE)
    print(f"import: {time.time()-t:.1f}s")
    page.wait_for_timeout(3000)
    for v in VIEWS:
        page.evaluate("window.__lt = 0; window.__ltMax = 0")
        t = time.time()
        page.evaluate(f"location.hash = '#{v}'")
        page.wait_for_timeout(200)
        page.evaluate(SETTLE)
        dt = time.time() - t
        lt = page.evaluate("[window.__lt, window.__ltMax]")
        print(f"{v:<12} {dt:6.2f}s  blocked {lt[0]/1000:5.2f}s (longest {lt[1]/1000:4.2f}s)")
    # person profile + edit + search typing
    pid = page.evaluate("""async () => { location.hash = '#/people'; return 1 }""")
    page.wait_for_timeout(1500)
    href = page.locator("a[href^='#/person/']:not([href='#/person/new'])").first.get_attribute("href")
    for v in [href[1:], href[1:] + "/edit"]:
        page.evaluate("window.__lt = 0")
        t = time.time()
        page.evaluate(f"location.hash = '#{v}'")
        page.wait_for_timeout(200)
        page.evaluate(SETTLE)
        print(f"{v:<12} {time.time()-t:6.2f}s  blocked {page.evaluate('window.__lt')/1000:5.2f}s")
    page.evaluate("location.hash = '#/people'")
    page.wait_for_timeout(1500)
    page.evaluate("window.__lt = 0")
    t = time.time()
    page.locator("input.q").type("Kowal", delay=80)
    page.evaluate(SETTLE)
    print(f"typing search  {time.time()-t:6.2f}s  blocked {page.evaluate('window.__lt')/1000:5.2f}s")
    page.screenshot(path="perf-people.png")
    page.evaluate("location.hash = '#/tree'")
    page.wait_for_timeout(4000)
    page.screenshot(path="perf-tree.png")
    print("errors:", errors[:5])
    b.close()
