/* Praetor site — shared behavior
   - highlight.js syntax highlighting
   - copy-to-clipboard on code blocks
   - scrollspy + mobile drawer for the guide sidebar
*/
(function () {
  "use strict";

  // --- syntax highlighting --------------------------------------------------
  if (window.hljs) {
    document.querySelectorAll("pre code").forEach(function (block) {
      window.hljs.highlightElement(block);
    });
  }

  // --- copy to clipboard ----------------------------------------------------
  document.querySelectorAll(".copy-btn").forEach(function (btn) {
    btn.addEventListener("click", function () {
      var block = btn.closest(".codeblock");
      var code = block && block.querySelector("code");
      if (!code) return;
      var text = code.innerText;

      var done = function () {
        btn.textContent = "Copied!";
        btn.classList.add("copied");
        setTimeout(function () {
          btn.textContent = "Copy";
          btn.classList.remove("copied");
        }, 1500);
      };

      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done).catch(fallback);
      } else {
        fallback();
      }

      function fallback() {
        var ta = document.createElement("textarea");
        ta.value = text;
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        try { document.execCommand("copy"); done(); } catch (e) { /* noop */ }
        document.body.removeChild(ta);
      }
    });
  });

  // --- guide sidebar: scrollspy + mobile drawer -----------------------------
  var toc = document.getElementById("toc");
  if (!toc) return;

  var links = Array.prototype.slice.call(toc.querySelectorAll("a"));
  var sidebar = document.getElementById("sidebar");
  var scrim = document.getElementById("scrim");
  var menuBtn = document.getElementById("menuBtn");

  var byId = {};
  links.forEach(function (a) {
    var id = a.getAttribute("href").slice(1);
    var target = document.getElementById(id);
    if (target) byId[id] = { link: a, section: target };
  });

  // mobile drawer open/close
  function setDrawer(open) {
    if (!sidebar) return;
    sidebar.classList.toggle("open", open);
    if (menuBtn) menuBtn.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (menuBtn) menuBtn.addEventListener("click", function () {
    setDrawer(!sidebar.classList.contains("open"));
  });
  if (scrim) scrim.addEventListener("click", function () { setDrawer(false); });

  // scrollspy: highlight the section currently in view
  var current = null;
  function setActive(id) {
    if (id === current || !byId[id]) return;
    current = id;
    links.forEach(function (a) { a.classList.remove("active"); });
    byId[id].link.classList.add("active");
  }

  var sections = Object.keys(byId).map(function (id) { return byId[id].section; });

  // the active section is the last one (in document order) whose top has
  // crossed the trigger line. Computing this directly from live layout, and
  // always taking the *last* qualifying section, means a section can't keep
  // winning just because it's tall enough for its tail to still poke past
  // the line while the next section has already started.
  var triggerLine = function () { return window.innerHeight * 0.35; };
  function computeActive() {
    // scrolled to the bottom: the last section may be too short for its top
    // to ever reach the trigger line, so just select it directly
    var doc = document.documentElement;
    if (window.innerHeight + window.scrollY >= doc.scrollHeight - 2) {
      return sections[sections.length - 1].id;
    }
    var line = triggerLine();
    var best = sections[0].id;
    for (var i = 0; i < sections.length; i++) {
      if (sections[i].getBoundingClientRect().top <= line) best = sections[i].id;
    }
    return best;
  }

  // clicking a link jumps straight to its section: select it immediately and
  // hold that choice until the smooth scroll settles, so the spy doesn't
  // read the still-mid-flight scroll position as some other section
  var lockUntil = 0;
  function spyActive() {
    if (Date.now() < lockUntil) return;
    setActive(computeActive());
  }
  links.forEach(function (a) {
    a.addEventListener("click", function () {
      var id = a.getAttribute("href").slice(1);
      if (byId[id]) { setActive(id); lockUntil = Date.now() + 1000; }
      if (window.matchMedia("(max-width: 900px)").matches) setDrawer(false);
    });
  });
  if ("onscrollend" in window) {
    window.addEventListener("scrollend", function () { lockUntil = 0; spyActive(); });
  }

  if ("IntersectionObserver" in window) {
    var observer = new IntersectionObserver(spyActive, { rootMargin: "0px 0px -65% 0px", threshold: [0, 1] });
    sections.forEach(function (s) { observer.observe(s); });
  } else {
    window.addEventListener("scroll", spyActive, { passive: true });
  }
  spyActive();

  // reflect the initial hash, if any
  if (location.hash && byId[location.hash.slice(1)]) {
    setActive(location.hash.slice(1));
  } else if (sections.length) {
    setActive(sections[0].id);
  }
})();
