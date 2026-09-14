// ============================================================
// TYLED
// Top-down 2D action / strategy
// ------------------------------------------------------------
// NES palette (square, triangle, sawtooth, 808, filtered white
// noise). ROBOT BOSS FIGHT theme: E minor, hammered
// repeated-note bass, biting syncopated square lead. Original
// material, three movements.
// ------------------------------------------------------------
// MOVEMENT 1 : players far apart  -> stalking / observation
// MOVEMENT 2 : players closing in -> tension / combat
// MOVEMENT 3 : endgame            -> the kill
//
// The "movement" control at top switches parts. Every switch
// leaving/entering the stake out goes through a BRIDGE (see
// bottom of file):
//   0 -> 1, 0 -> 2 : ACCELERATING, 6 bars — machine spools up
//            (hats 2->16/bar, bass 4->12). DESTINATION-AGNOSTIC:
//            "STALK spooling up", not "becoming TENSION" —
//            lands wherever the slider reads after six bars.
//   1 -> 0, 2 -> 0 : DECELERATING, 4 bars — winds down (hats
//            8->2, bass 12->4). Starts one step below the
//            climb's top since it starts from TENSION (hats 8),
//            not the bridge's peak: 2->0 reads "drop to fighting
//            density, then wind down".
// Only 1 <-> 2 stays immediate: both already at fighting
// density, nothing to spool.
// ============================================================


// ============================================================
// CONTROL PANEL
// ============================================================

// 0 = STALK
// 1 = TENSION
// 2 = ENDGAME

const movement = slider(0, 0, 2, 1)


// Master volume
const volume = slider(0.8, 0, 1, 0.01)


// Strategic filter: 0 = closed/muffled/far, 1 = open/bright/close.
// Acts on the WHOLE mix, drums included — the track has to be
// killable with a single gesture.
//
// So no layer sets its own .lpf() (would be overwritten).
// Timbral contrast comes from WAVEFORM (triangle -> sawtooth)
// and envelopes instead. White noise .hpf() survives — different
// param.
const filterOpenness = slider(0.45, 0, 1, 0.01)


// ============================================================
// GLOBAL SETTINGS
//
// CAREFUL: setcpm counts CYCLES per minute, not quarter notes.
// ONE CYCLE = ONE BAR of four beats at 160 BPM: 160/4 = 40
// cycles/min, 1.5 s/bar. 160 not 150: sixteenth = 94 ms, tight
// enough for the bass to read as an engine, not a walk.
//
// Phrase is always 8 bars, written as a slowcat < ... > (8 bars
// = 8 cycles) — natural Strudel spelling, and what makes
// movement switching fast (see bottom of file).
//
// So every melodic line is < [bar1] ... [bar8] >, 8 eighth-note
// slots/bar (a slot may split into sixteenths: [x y]). Drums are
// written over ONE bar and repeat themselves: NO .fast(8).
// ============================================================

setcpm(160 / 4)


// ============================================================
// THE MOTIF — "THE BITE"
//
// One three-note gesture drives the track:
//     E - E - B      (repeated note, then a FALL of a fourth)
// A fall reads as a jaw closing (vs a leap-up "hop", playful).
// Bare and spaced in movement 1, hammered sixteenths in
// movement 2, screamed an octave higher in movement 3.
//
// All three movements share tempo, key (E MINOR) and 8-bar grid:
//     Em - Em - C - D - Am - C - B - Em
//      i    i   VI  VII  iv  VI  V   i
// This is what makes switches seamless: bar 5 of TENSION
// harmonically continues bar 4 of STALK.
//
// B MAJOR (bar 7, D sharp, borrowed from harmonic minor) is the
// only foreign note — the leading tone into bar 8's E. Harmless
// between movements (a switch always lands on the same bar both
// sides), but forbidden at the accelerating bridge's EXIT: see
// that section.
// ============================================================


// ============================================================
// MOVEMENT 1 : STALK
// Robots far apart. Bite stated bare, low, lots of silence — not
// calm, a stake out. Bass on a slow pulse (4 hits/bar), drone of
// open fifths (never a third: too friendly).
// Grid: Em - Em - C - D - Am - C - B - Em.
// ============================================================

const melodyStalk =
  note(`<
    [e4  e4  ~   b3  ~   ~   e4  ~  ]
    [~   e4  ~   g4  f#4 ~   e4  ~  ]
    [g4  g4  ~   e4  ~   ~   c4  ~  ]
    [~   d4  ~   f#4 a4  ~   f#4 ~  ]
    [a4  a4  ~   e4  ~   ~   c5  ~  ]
    [~   g4  ~   e4  g4  ~   b4  ~  ]
    [f#4 f#4 ~   d#4 ~   ~   b4  ~  ]
    [~   e5  ~   d5  b4  ~   e4  ~  ]
  >`)
    .sound("square")
    .decay(.12)
    .sustain(.2)
    // Long release + reverb/echo: tail turns a movement switch
    // into a fade (see transitions note at bottom). Echo tuned to
    // the eighth note (0.1875 s at 160 BPM): fills silences
    // instead of smearing them.
    .release(.3)
    .room(.26)
    .roomsize(2)
    .delay(.24)
    .delaytime(.1875)
    .delayfeedback(.32)
    .gain(.5)

const bassStalk =
  note(`<
    [e1 ~  e2 ~  e1 ~  b1 ~ ]
    [e1 ~  e2 ~  d2 ~  b1 ~ ]
    [c2 ~  c3 ~  g1 ~  c2 ~ ]
    [d2 ~  d3 ~  a1 ~  d2 ~ ]
    [a1 ~  a2 ~  e2 ~  a1 ~ ]
    [c2 ~  c3 ~  g1 ~  b1 ~ ]
    [b1 ~  b2 ~  f#2 ~ d#2 ~]
    [e1 ~  e2 ~  b1 ~  e2 ~ ]
  >`)
    // Triangle: softest rung of the timbre ladder
    // (triangle -> sawtooth); teeth come from TENSION onward.
    .sound("triangle")
    .decay(.3)
    .sustain(.2)
    // Short release: a bass tail would muddy the switch.
    .release(.12)
    .gain(.72)

// Drone of open fifths, buried in the mix: states the grid
// without pulling focus; no third means it can't turn major.
const droneStalk =
  note(`<
    [e3,b3] [e3,b3] [c3,g3] [d3,a3]
    [a2,e3] [c3,g3] [b2,f#3] [e3,b3]
  >`)
    .sound("triangle")
    // Soft attack: swells in, never clicks, even via a switch.
    .attack(.18)
    .decay(.5)
    .sustain(.4)
    .release(.6)
    .room(.35)
    .gain(.16)

const drumsStalk =
  stack(
    s("bd ~ bd ~, ~ ~ sd ~, hh ~ hh ~")
      .bank("RolandTR808")
      .gain(.5),

    // A breath of noise every other bar, just to inhale.
    s("<~ [~ ~ ~ ~ ~ ~ ~ white]>")
      .hpf(9000)
      .decay(.02)
      .gain(.2)
  )

const movementStalk =
  stack(melodyStalk, bassStalk, droneStalk, drumsStalk)


// ============================================================
// MOVEMENT 2 : TENSION
// Robots closing in. Bite goes to sixteenths, repeats without a
// breath: hammer, hammer, fall, riposte. Bass switches to
// sawtooth, alternates low/high octave across the eight eighths
// — energy from that, not volume. Lead's only rest in all eight
// bars sits at the very end of bar 8, its sole run-up.
// ============================================================

const melodyTension =
  note(`<
    [[e5 e5]   b4  [e5 e5]  b4  [g5 f#5] e5  [d5 e5]  b4 ]
    [[e5 e5]   b4  [e5 e5]  g5  [f#5 e5] d5  [e5 f#5] g5 ]
    [[g5 g5]   e5  [g5 g5]  e5  [c6 b5]  g5  [a5 g5]  e5 ]
    [[f#5 f#5] d5  [f#5 a5] d6  [c6 a5]  f#5 [d5 e5]  f#5]
    [[a5 a5]   e5  [a5 a5]  e5  [c6 b5]  a5  [g5 a5]  e5 ]
    [[g5 g5]   c6  [b5 g5]  e5  [g5 a5]  b5  [c6 b5]  g5 ]
    [[f#5 f#5] d#5 [f#5 b5] f#5 [a5 g5]  f#5 [d#5 f#5] a5]
    [[e5 e5]   b5  [g5 e5]  b4  [e5 f#5] g5  b5       ~  ]
  >`)
    .sound("square")
    .decay(.07)
    .sustain(.12)
    // Shorter than movement 1 (rate doubled), but never below
    // ~0.15 s or the tail stops covering the switch and pick
    // becomes an audible cut.
    .release(.18)
    .room(.14)
    .roomsize(1.5)
    .gain(.6)

const bassTension =
  note(`<
    [e1 e2 e1 e2 e1 e2 b1  d2 ]
    [e1 e2 e1 e2 g1 b1 e1  e2 ]
    [c2 c3 c2 c3 g1 c2 e2  g2 ]
    [d2 d3 d2 d3 a1 d2 f#2 a2 ]
    [a1 a2 a1 a2 e2 a1 c2  e2 ]
    [c2 c3 c2 c3 g1 c2 g2  c3 ]
    [b1 b2 b1 b2 f#2 b1 d#2 f#2]
    [e1 e2 e1 e2 b1 e2 d2  c2 ]
  >`)
    // Sawtooth: the timbre rung separating observation from
    // combat. No distortion in this palette — grit comes from
    // waveform, short decay, and gain instead.
    .sound("sawtooth")
    .decay(.13)
    .sustain(.08)
    .release(.05)
    .gain(.78)

const drumsTension =
  stack(
    s("bd ~ bd bd, ~ sd ~ sd, hh*8")
      .bank("RolandTR808")
      .gain(.75),

    // Snare pickup on the 4th bar of each four. One cycle = one
    // bar, so this 4-element slowcat lands every 4 bars (bars 4
    // and 8 of the phrase).
    s("<~!3 [~ ~ sd sd]>")
      .bank("RolandTR808")
      .gain(.55),

    s("white*4")
      .hpf(9000)
      .decay(.02)
      .gain(.26)
  )

const movementTension =
  stack(melodyTension, bassTension, drumsTension)


// ============================================================
// MOVEMENT 3 : ENDGAME
// Bite screamed an octave higher, bass to sixteenths on the
// repeated note (12 attacks/bar vs 8), second square voice
// hammers a syncopated 3+3+2 pedal. Bar 7: lead, pedal and bass
// all spell the leading tone (D sharp) together.
// ============================================================

const melodyEndgame =
  note(`<
    [[e6 e6]   b5  [e6 e6]  b5  [g5 f#5] e5  [b5 d6]  e6 ]
    [[e6 e6]   d6  [b5 g5]  e5  [f#5 g5] b5  [d6 e6]  f#6]
    [[g5 g5]   e6  [c6 g5]  e5  [g5 c6]  e6  [g6 e6]  c6 ]
    [[f#6 f#6] d6  [a5 f#5] d5  [f#5 a5] d6  [f#6 d6] a5 ]
    [[a5 a5]   e6  [a5 a5]  e6  [c6 b5]  a5  [e6 c6]  a5 ]
    [[c6 c6]   g5  [c6 e6]  g6  [e6 c6]  g5  [b5 c6]  e6 ]
    [[f#6 f#6] d#6 [b5 f#5] d#5 [f#5 a5] b5  [d#6 f#6] a6]
    [[e6 e6]   b5  [g5 e5]  b5  [e6 d6]  b5  [g5 b5]  e6 ]
  >`)
    .sound("square")
    .decay(.06)
    .sustain(.1)
    .release(.16)
    .room(.14)
    .roomsize(1.5)
    // ENDGAME is the densest state, sets the mix ceiling. Bus
    // peaks (temporary .analyze() on the final chain, volume
    // slider at max): STALK 0.65, TENSION 0.90, ENDGAME 0.99.
    // All six gains here trimmed to that ceiling — "natural"
    // values clipped at 1.10 on the downbeats. Ladder must stay
    // ENDGAME > TENSION > STALK.
    .gain(.55)

// Second voice: syncopated 3+3+2 pedal, tightens the rhythm
// against the bass rather than accompanying it.
const counterEndgame =
  note(`<
    [e4 ~ ~ e4 ~ ~ b4  ~]
    [e4 ~ ~ e4 ~ ~ g4  ~]
    [c4 ~ ~ c4 ~ ~ g4  ~]
    [d4 ~ ~ d4 ~ ~ a4  ~]
    [a3 ~ ~ a3 ~ ~ e4  ~]
    [c4 ~ ~ c4 ~ ~ e4  ~]
    [b3 ~ ~ b3 ~ ~ d#4 ~]
    [e4 ~ ~ b3 ~ ~ e4  ~]
  >`)
    .sound("square")
    .decay(.05)
    .sustain(0)
    .release(.13)
    .room(.12)
    .gain(.25)

const bassEndgame =
  note(`<
    [[e1 e1] e1 [e1 e2] e1 [e1 e1] e2 [b1 e2]  e1 ]
    [[e1 e1] e1 [e1 e2] e1 [e1 e1] e2 [d2 e2]  g1 ]
    [[c2 c2] c2 [c2 c3] c2 [c2 c2] c3 [g1 c3]  c2 ]
    [[d2 d2] d2 [d2 d3] d2 [d2 d2] d3 [c3 b2]  a1 ]
    [[a1 a1] a1 [a1 a2] a1 [a1 a1] a2 [e2 a2]  a1 ]
    [[c2 c2] c2 [c2 c3] c2 [c2 c2] c3 [e2 g2]  c3 ]
    [[b1 b1] b1 [b1 b2] b1 [b1 b1] b2 [f#2 b2] d#2]
    [[e1 e1] e1 [e1 e2] e1 [e1 e1] e2 [b1 d2]  c2 ]
  >`)
    .sound("sawtooth")
    .decay(.1)
    .sustain(.06)
    .release(.04)
    .gain(.66)

const drumsEndgame =
  stack(
    s("bd bd bd bd, ~ sd ~ sd, hh*16")
      .bank("RolandTR808")
      .gain(.72),

    // Full roll every 4 bars, pushing toward the end.
    s("<~!3 [sd sd sd sd]>")
      .bank("RolandTR808")
      .gain(.44),

    s("white*8")
      .hpf(9000)
      .decay(.015)
      .gain(.22)
  )

const movementEndgame =
  stack(melodyEndgame, counterEndgame, bassEndgame, drumsEndgame)


// ============================================================
// ACCELERATING BRIDGE  (STALK -> TENSION / ENDGAME)
//
// A straight cut from stake out to fight was too brutal; we want
// to HEAR the machine spool up. 6 bars (9 s), accelerating one
// thing everywhere: SUBDIVISION.
//
// Onsets per bar (STALK/TENSION for reference):
//
//   bar       hats  kick  snare  noise  bass  lead  wave
//   STALK       2     2     1     0/1     4     4   tri
//   bridge 1    2     2     0      0      4     4   tri
//   bridge 2    4     2     0      1      4     6   tri
//   bridge 3    4     2     1      2      6     8   tri
//   bridge 4    6     2     2      2      8    10   SAW
//   bridge 5    8     3     2      4      8    12   saw
//   bridge 6   16     4     4      8     12    16   saw
//   TENSION     8     3     2      4      8    12   saw
//
// Bar 6 deliberately overshoots TENSION — redline before the
// drop, not a preview of it.
//
// GRID: Em - Am - C - D - C - E5.
//
// LAST BAR IS AN OPEN FIFTH (E/B, no third), not a B7 dominant:
// bridge is 6 bars, grid is 8, so TENSION resumes at bar
// (start+6) mod 8 — any bar. B7 ends on D SHARP, risking a cross
// relation with bar 4's D NATURAL bass. Open E/B is consonant
// with all five grid chords, and reads as "engine redlining"
// rather than a cadence.
//
// EXITING INTO ENDGAME (0 -> 2) needs no new material, fits
// TIGHTER than the TENSION exit:
//
//   line         bridge 6   ENDGAME
//   hats            16        16
//   kick             4         4
//   snare            4         2  (4 on the roll bar)
//   noise            8         8
//   bass            12        12
//   lead            16        12
//   skins gain    .72       .72   (TENSION's kick is .75)
//
// Bar 6 mostly LANDS into ENDGAME rather than overshooting — only
// snare and lead still redline. Register agrees (bar 6 sits in
// ENDGAME's octaves 5-6) and only spells E/B, so the cross
// relation above can't bite. Bass already sawtooth both sides.
//
// Each bridge bar is a ONE-CYCLE pattern (not a slowcat), picked
// via `bridgeBar`: plays identically at any absolute cycle
// number, so the bridge always unfolds 1-6 wherever it starts.
//
// WRITING RULE: every bar needs an EVENT ON ITS DOWNBEAT, on
// every layer — a step change retriggers the previous step's
// head note once (see decelerating bridge), inaudible only if
// something masks it at the same instant.
// ============================================================

const BRIDGE_UP_BARS = 6


// Switch memory: the ONLY mutable state in the track (Strudel has
// no "previous slider value" of its own).
//
// -Infinity = "bridge not armed" — not Infinity. With -Infinity
// both progress tests (x < start, x - start < LENGTH) read false,
// so the requested state applies IMMEDIATELY. With Infinity,
// x < start was always true and an arrival into TENSION not from
// STALK (2 -> 1) stuck on STALK forever (fixed bug).
//
// Three things depend on that sentinel:
//  - the instant switches above;
//  - the climb's `else e = mv` exit, letting the accelerating
//    bridge hand off to ENDGAME too, not just TENSION;
//  - _bridgeFrom, read only under `x < _bridgeDownStart` — dead
//    code while the sentinel is -Infinity, so it needs no
//    "nothing pending" value.
//
// _bridgeFrom remembers WHICH movement the decelerating bridge
// is leaving (1 or 2), so the bar fragment before its downbeat
// finishes in the outgoing movement.
//
// _prevState starts at 0: a slider already on 1/2 at load makes
// the track open through the accelerating bridge instead of
// straight into TENSION/ENDGAME — intended, since the game always
// starts in STALK and the REPL wants to hear the spool-up.
let _prevSlider = null
let _prevState = 0
let _bridgeUpStart = -Infinity
let _bridgeDownStart = -Infinity
let _bridgeFrom = 0


// Reads the current slider value. `movement` is a Pattern:
// queried over a tiny span, live, not frozen at cycle start.
const readMovement = (t) => {
  const e = movement.queryArc(t, t + 1e-6)
  return e.length ? e[0].value : 0
}


// Bar index inside the accelerating bridge: 0..5.
const bridgeUpBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeUpStart)
  return Math.min(BRIDGE_UP_BARS - 1, Math.max(0, i))
})


// STALK's bite eaten away by sixteenths until it becomes
// TENSION's hammer. Last bar: E and B, the unison.
const melodyBridgeUp =
  bridgeUpBar.pick([
    note("[e5 e5 ~  b4 ~  ~  e5 ~ ]"),
    note("[a4 a4 ~  e5 [d5 c5] ~  a4 ~ ]"),
    note("[c5 c5 ~  g5 [e5 d5] c5 [b4 c5] ~ ]"),
    note("[d5 [d5 e5] f#5 [e5 d5] a5 [f#5 g5] a5 ~ ]"),
    note("[[c5 d5] e5 [d5 c5] g5 [e5 f#5] g5 [a5 g5] e5]"),
    note("[[e5 e5] [e5 e5] [b5 b5] [e6 e6] [e5 e5] [b5 b5] [e6 b5] [e6 e6]]")
  ])
    .sound("square")
    .decay(.09)
    .sustain(.16)
    .release(.22)
    .room(.18)
    .roomsize(1.6)
    .gain(.58)


// Bass walks the same road: pulse, eighths, sixteenths. Timbre
// climbs with it — triangle for bars 1-3, sawtooth from bar 4.
// .sound() is carried by EACH entry, not applied after the pick
// (which would overwrite it).
const bassBridgeUp =
  bridgeUpBar.pick([
    note("[e1 ~  e2 ~  e1 ~  b1 ~ ]").sound("triangle"),
    note("[a1 ~  a2 ~  e2 ~  a1 ~ ]").sound("triangle"),
    note("[c2 ~  c3 c2 g1 ~  c3 c2]").sound("triangle"),
    note("[d2 d3 d2 d3 a1 d2 f#2 a2]").sound("sawtooth"),
    note("[c2 c3 c2 c3 g1 c2 e2 g2]").sound("sawtooth"),
    note("[[e1 e1] e2 [e1 e1] e2 [e1 e1] e2 [b1 e2] e2]").sound("sawtooth")
  ])
    .decay(.2)
    .sustain(.12)
    .release(.07)
    .gain(.76)


// Builds the drum kit for one bridge bar. .bank() stays on the
// 808 layer only — white noise isn't part of that bank.
//
// `att` attenuates all three layers at once: 1 through the
// accelerating bridge (acceleration alone raises pressure),
// decreasing bar by bar in the decelerating bridge to land on
// STALK's level with no step. Pinned to real drum gains:
// .72*1 ~= .75 (TENSION's kick) at top, .72*.7 ~= .50 (STALK's
// kick) at bottom — change a movement's drum gain, redo this
// arithmetic. ENDGAME needs no arithmetic of its own: its kick
// is .72, exactly `.72*1`, so both bridges land on/leave it
// cleanly. A 2 -> 0 downbeat is thus a DENSITY step (hats 16->8)
// with no volume step — the point of the manoeuvre.
const bridgeDrums = (skins, hats, noise, att = 1) =>
  stack(
    s(skins).bank("RolandTR808").gain(.72 * att),
    s(hats).bank("RolandTR808").gain(.5 * att),
    s(noise).hpf(9000).decay(.02).gain(.26 * att)
  )

// Acceleration is most audible here: hats 2 -> 16/bar, kick
// fills in, snare arrives and rolls.
const drumsBridgeUp =
  bridgeUpBar.pick([
    bridgeDrums("bd ~  bd ~ ",             "hh ~ hh ~", "~"),
    bridgeDrums("bd ~  ~  bd",             "hh*4",      "white"),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white*2"),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2"),
    bridgeDrums("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4"),
    bridgeDrums("bd bd bd bd, sd*4",       "hh*16",     "white*8")
  ])

const movementBridgeUp =
  stack(melodyBridgeUp, bassBridgeUp, drumsBridgeUp)


// ============================================================
// DECELERATING BRIDGE  (TENSION / ENDGAME -> STALK)
//
// Mirror of the previous bridge. TENSION straight into STALK
// read as a breakdown, not a release — a climb with no descent.
// So we wind SUBDIVISION back down.
//
// Onsets per bar:
//
//   bar       hats  kick  snare  noise  bass  lead  wave
//   TENSION     8     3     2      4      8    12   saw
//   bridge 1    8     3     2      4     12    12   saw
//   bridge 2    6     2     2      2      8     8   saw
//   bridge 3    4     2     1      1      6     5   TRI
//   bridge 4    2     2     1      1      4     4   tri
//   STALK       2     2     1     0/1     4     4   tri
//
// Bar 1 keeps TENSION's hats but pushes BASS to 12: the fall
// starts by leaning forward, a gear change before a drop in
// energy. Bar 4 matches STALK line for line — only the
// open-fifth drone is missing, swelling in next bar.
//
// ENTERING FROM ENDGAME (2 -> 0): table starts from TENSION, and
// ENDGAME gets that too — bar 1's hats (8) against ENDGAME's 16
// make the downbeat an audible density step, mirroring the
// climb's 0 -> 2. Nothing steps in VOLUME (bar 1 skins .72*1 =
// ENDGAME's kick .72).
//
// Melody also descends in REGISTER bar after bar (octave 5-6 ->
// 4), reverse of the climb, audible even without the hats.
//
// WHY 4 BARS NOT 6: a climb is waited for, a fall is felt. Six
// steps down would force repeated densities (8-6-6-4-4-2) and
// drag. Four bars = 6 s, one hat step/bar (8-6-4-2), every step
// audible.
//
// GRID: Am - C - D - Em (iv - VI - VII - i). The accelerating
// bridge ends on an open fifth so it can land on any grid bar;
// here we want STALK ALREADY settled, so the cadence closes
// inside the bridge on bar 4's E. Pick preserves phase, so any
// chord after an E minor continues naturally.
//
// Bar 4 literally quotes STALK bar 1 (melody E-E-B, bass E/B,
// "bd ~ bd ~ / hh ~ hh ~"): the bridge becomes the next movement.
// ============================================================

const BRIDGE_DOWN_BARS = 4


// Bar index inside the decelerating bridge: 0..3.
const bridgeDownBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeDownStart)
  return Math.min(BRIDGE_DOWN_BARS - 1, Math.max(0, i))
})


// TENSION's hammer un-syncopates: sixteenths drop away until
// only the bare bite remains, line falling an octave and a half.
const melodyBridgeDown =
  bridgeDownBar.pick([
    note("[[a5 a5] e5 [a5 g5] e5 [c6 b5] a5 [g5 a5] e5]"),
    note("[c5 [e5 d5] c5 ~  g4 [c5 b4] g4 ~ ]"),
    note("[d5 d5 ~  a4 f#4 ~  d4 ~ ]"),
    note("[e4 e4 ~  b3 ~  ~  e4 ~ ]")
  ])
    .sound("square")
    .decay(.12)
    .sustain(.18)
    // Longer release than the accelerating bridge: tail builds
    // the fade into STALK, where the climb needed to stay crisp.
    .release(.3)
    .room(.24)
    .roomsize(1.8)
    .gain(.54)


// Bass walks bassBridgeUp's road backwards: sixteenths, eighths,
// then STALK's pulse — timbre comes down too, sawtooth -> triangle.
const bassBridgeDown =
  bridgeDownBar.pick([
    note("[[a1 a1] a2 [a1 a1] a2 [a1 a1] a2 [e2 a2] a1]").sound("sawtooth"),
    note("[c2 c3 c2 c3 g1 c2 e2 g2]").sound("sawtooth"),
    note("[d2 ~  d3 d2 a1 ~  d3 d2]").sound("triangle"),
    note("[e1 ~  e2 ~  e1 ~  b1 ~ ]").sound("triangle")
  ])
    .decay(.24)
    .sustain(.14)
    .release(.1)
    .gain(.72)


// Hats come down 8 -> 6 -> 4 -> 2, kick empties out, snare fades,
// attenuation follows the density drop to land on STALK's level.
//
// On a downbeat where the step changes, the PREVIOUS step's note
// still triggers once on top of the new one (pick samples just
// before the bar line) — true of the accelerating bridge too, but
// inaudible there since both hits share gain. Here gains differ,
// so the doubled hit is audible but lands on the downbeat, where
// the kick masks it. Nothing to fix, but it's why the WRITING
// RULE above matters: a bar starting on a rest would let it
// bleed through in the open.
const drumsBridgeDown =
  bridgeDownBar.pick([
    bridgeDrums("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4", 1),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2", .92),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white",   .82),
    bridgeDrums("bd ~  bd ~ , ~ ~  sd ~ ", "hh ~ hh ~", "white",   .7)
  ])

const movementBridgeDown =
  stack(melodyBridgeDown, bassBridgeDown, drumsBridgeDown)


// ============================================================
// STATE MACHINE
//
// THE STATE ACTUALLY PLAYING: 0/1/2 like the slider, plus 3
// (accelerating bridge) and 4 (decelerating bridge).
//
// signal(fn) receives time in cycles, letting us count bars
// since the switch. As a pick selector it samples at bar start,
// so bridge steps land exactly on downbeats.
//
// Math.ceil pins the bridge start to the NEXT BAR so the current
// bar finishes cleanly first — without it, steps land mid-bar.
//
// ARMING: a bridge arms only if the OUTGOING STATE suits it —
// STALK (0) for accelerating, TENSION/ENDGAME (1/2) for
// decelerating — never the SLIDER value: between the gesture and
// Math.ceil there's a bar-fragment where the old movement still
// sounds but the slider already reads new. A fast 0->1->0 round
// trip in that window would otherwise arm a decelerating bridge
// out of a texture that never accelerated. _prevState fixes this
// — any round trip inside a bridge or its waiting window disarms
// both and switches cleanly.
//
// DESTINATION-AGNOSTIC CLIMB: "STALK spooling up", not "STALK
// becoming TENSION" — arms for ANY non-zero slider from a played
// STALK, exits via `e = mv` to whatever the slider reads after
// six bars. 0->2 gets the same ramp as 0->1; swapping the
// destination mid-climb RETARGETS the landing rather than
// aborting (ramp keeps all six bars).
//
// WHY THE RETARGET TEST READS THE BRIDGE CLOCK, NOT _prevState:
// _prevState is rewritten on EVERY query (lookahead included), so
// at a slider gesture it may hold a state not yet audible —
// testing it for "am I in a bridge" caused this exact ordering
// bug before. `x - _bridgeUpStart < BRIDGE_UP_BARS` reads the
// bridge's own clock, which only moves at arming time. The two
// remaining _prevState tests are safe: they check SETTLED states
// (0/1/2), steady across a whole bar.
//
// Returning to a bridge's own start value still aborts it, for a
// different reason each side of the downbeat:
//  - in the waiting window (x < _bridgeUpStart), _prevState is
//    still 0, so mv === 0 fails both arm guards and STALK
//    continues;
//  - once sounding, _prevState is 3, so mv === 0 fails the
//    retarget test and 3 matches neither bridge's arm condition.
// Either way, no bridge feeds straight into the other.
const playedState = signal((t) => {
  const x = Number(t)
  const mv = readMovement(x)

  if (mv !== _prevSlider) {
    if (mv !== 0 && x - _bridgeUpStart < BRIDGE_UP_BARS) {
      // Accelerating bridge armed or already running. It does
      // not care where it lands, so leave it be: it retargets.
    } else if (mv !== 0 && _prevState === 0) {
      _bridgeUpStart = Math.ceil(x)                   // STALK -> TENSION / ENDGAME
      _bridgeDownStart = -Infinity
      _bridgeFrom = 0
    } else if (mv === 0 && (_prevState === 1 || _prevState === 2)) {
      _bridgeDownStart = Math.ceil(x)                 // TENSION / ENDGAME -> STALK
      _bridgeUpStart = -Infinity
      _bridgeFrom = _prevState
    } else {
      _bridgeUpStart = -Infinity                      // everything else: instant
      _bridgeDownStart = -Infinity
    }
    _prevSlider = mv
  }

  let e
  if (mv === 0) {
    if (x < _bridgeDownStart) e = _bridgeFrom                // finish current bar
    else if (x - _bridgeDownStart < BRIDGE_DOWN_BARS) e = 4  // decelerating bridge
    else e = 0                                               // STALK settled
  } else {
    if (x < _bridgeUpStart) e = 0                            // finish current bar
    else if (x - _bridgeUpStart < BRIDGE_UP_BARS) e = 3      // accelerating bridge
    else e = mv                                              // TENSION / ENDGAME settled
  }

  _prevState = e
  return e
})


// ============================================================
// MOVEMENT SELECTION  —  pick, NOT pickRestart
//
// This decides how smooth the transitions are.
//
// WHY NOT pickRestart: restarts the chosen pattern every cycle,
// so a slowcat phrase plays bar 1 forever (e4 e4 e4...). An old
// version worked around this by cramming all 8 bars into one
// 12 s cycle, making switches take up to 12 s.
//
// pick restarts nothing — routes to the chosen pattern, which
// keeps running on ITS OWN clock:
//  - Phase preserved: switching on bar 5 enters the new movement
//    at ITS bar 5. Shared grid means harmony continues with no
//    seam — the real source of the smoothness.
//  - Structure preserved: pick takes structure from the chosen
//    pattern, not the selector, so each bar's 8 eighths stay
//    intact.
//
// SWITCH LATENCY: slider() reads live at query time, not frozen
// at cycle start. Instant switches take effect on the next
// scheduled note — one eighth, ~0.19 s at 160 BPM.
//
// THE DELIBERATE EXCEPTIONS: every switch leaving/entering the
// stake out goes through a bridge.
//   0 -> 1, 0 -> 2 : accelerating, 6 bars (9 s). Heard right away
//            (next bar, 1.5 s), full accelerate takes ~10.5 s.
//            0 -> 2 is the same climb, landing on ENDGAME.
//   1 -> 0, 2 -> 0 : decelerating, 4 bars (6 s), ~7.5 s total —
//            shorter on purpose (see that section). From ENDGAME
//            the wind-down opens on TENSION's density (hats
//            16 -> 8 on the downbeat): 2 -> 0 chains through 1.
// Only 1 <-> 2 stays instant, both directions.
//
// Moving the slider mid-bridge aborts it, except retargeting a
// running accelerating bridge's DESTINATION (1 <-> 2), which
// keeps the whole ramp. Returning to the bridge's own start value
// still aborts; no bridge feeds straight into the other.
//
// WHY IT DOES NOT CLICK: pick interrupts no sounding voice, only
// stops producing new ones — triggered notes finish their
// envelope. Melodic layers keep release >= ~0.15 s plus reverb so
// the tail crossfades into the incoming movement. Bass and drums
// stay dry — a low tail would muddy the switch.
//
// Two syntax traps:
//  - Argument order: it's selector.pick([...]). The reverse
//    (stack(...).pick(selector)) loops infinitely at evaluation.
//  - slider() returns a Pattern, not a number — raw arithmetic
//    on it silently yields NaN then an AudioParam error. Always
//    go through .range(min, max).
//
// Finally .mul(gain(volume)), not .gain(volume): .gain()
// overwrites each layer's own gain (mix disappears), .mul()
// multiplies it.
// ============================================================

// Array order IS the state encoding: 0 STALK, 1 TENSION,
// 2 ENDGAME, 3 accelerating bridge, 4 decelerating bridge. pick
// is positional and silent if off by one — an entry inserted
// mid-array would play the wrong movement with no error.
playedState
  .pick([
    movementStalk,
    movementTension,
    movementEndgame,
    movementBridgeUp,
    movementBridgeDown
  ])
  .lpf(filterOpenness.range(500, 12000))
  .mul(gain(volume))
