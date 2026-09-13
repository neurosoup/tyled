// ============================================================
// TYLED
// Top-down 2D action / strategy
// ------------------------------------------------------------
// Same NES palette as before (square, triangle, sawtooth, 808,
// filtered white noise), but a ROBOT BOSS FIGHT theme:
// E minor, hammered repeated-note bass, biting syncopated
// square lead. Original material, told in three movements.
// ------------------------------------------------------------
// MOVEMENT 1 : players far apart  -> stalking / observation
// MOVEMENT 2 : players closing in -> tension / combat
// MOVEMENT 3 : endgame            -> the kill
//
// The "movement" control at the top switches parts.
// Every switch that LEAVES or ENTERS the stake out goes
// through a BRIDGE (see the bottom of the file):
//   0 -> 1  and  0 -> 2 : ACCELERATING BRIDGE, 6 bars, the
//            machine spools up (hats 2 -> 16 per bar, bass
//            4 -> 12). The climb is DESTINATION-AGNOSTIC: it
//            is "STALK spooling up", not "STALK becoming
//            TENSION", so it lands on whatever the slider
//            reads when its six bars are up.
//   1 -> 0  and  2 -> 0 : DECELERATING BRIDGE, 4 bars, the
//            machine winds back down (hats 8 -> 2, bass
//            12 -> 4). It starts one step below the climb's
//            top because it starts from TENSION (hats 8), not
//            from the bridge's peak — so 2 -> 0 reads as
//            "drop to fighting density, then wind down".
// Only 1 <-> 2 stays immediate, in both directions: both ends
// are already at fighting density, there is nothing to spool.
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


// Strategic filter
// 0 = very closed (muffled, far away)
// 1 = very open (bright, right here)
// It acts on the WHOLE mix, drums included: that is on purpose,
// the track has to be killable with a single gesture.
//
// Corollary: NO layer sets its own .lpf(), it would be
// overwritten by this one. Timbral contrast therefore comes
// from the WAVEFORM (triangle -> sawtooth) and from envelopes,
// never from a local filter. The white noise .hpf() does
// survive: it is a different parameter.
const filterOpenness = slider(0.45, 0, 1, 0.01)


// ============================================================
// GLOBAL SETTINGS
//
// CAREFUL: setcpm counts CYCLES per minute, not quarter notes.
// Here ONE CYCLE = ONE BAR of four beats at 160 BPM, i.e.
// 160/4 = 40 cycles per minute, i.e. 1.5 s per bar.
//
// 160 and not 150: this track lives on hammered sixteenths,
// and 1.5 s per bar puts the sixteenth at 94 ms — tight enough
// for the bass to read as an engine rather than a walk.
//
// The full phrase is always 8 bars, but it is written as a
// slowcat < ... > : 8 bars = 8 cycles. That is the natural
// Strudel spelling, and it is what makes movement switching
// fast (see the bottom of the file).
//
// Practical consequence: every melodic line is written as
// < [bar1] [bar2] ... [bar8] >, 8 eighth-note slots per bar
// (a slot may split into two sixteenths: [x y]).
// Drums are written over ONE bar and repeat by themselves,
// one cycle = one bar: NO .fast(8).
// ============================================================

setcpm(160 / 4)


// ============================================================
// THE MOTIF — "THE BITE"
//
// The whole track rests on one three-note gesture:
//     E - E - B      (repeated note, then a FALL of a fourth)
//
// It is the exact inversion of the old version's "hop"
// (repeated note, then a leap upward): same gesture, opposite
// direction. A leap up sounds playful; a fall sounds like a
// jaw closing. It appears bare and spaced out in movement 1,
// hammered in sixteenths in movement 2, screamed an octave
// higher in movement 3.
//
// All three movements share the SAME tempo, the SAME key
// (E MINOR) and the SAME 8-bar grid:
//
//     Em - Em - C - D - Am - C - B - Em
//      i    i   VI  VII  iv  VI  V   i
//
// This is not a spelling detail: it is what makes the switches
// seamless, since bar 5 of TENSION harmonically continues bar
// 4 of STALK.
//
// The B MAJOR of bar 7 (D sharp, borrowed from harmonic minor)
// is the only foreign note in the track: it is the leading
// tone, it pulls toward the E of bar 8. It is harmless between
// movements — a switch always lands on the SAME bar on both
// sides, hence on the same chord. It is however forbidden at
// the EXIT of the accelerating bridge: see that section.
// ============================================================


// ============================================================
// MOVEMENT 1 : STALK
// The robots are far apart. The bite is stated bare, low, with
// a lot of silence around it: this is not calm, it is a stake
// out. Bass on a slow pulse (4 hits per bar), drone of open
// fifths (never a third: a third would sound friendly).
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
    // Long release plus a little reverb and echo: that tail is
    // what turns a movement switch into a fade (see the note on
    // transitions at the bottom).
    // The echo is tuned to the eighth note (0.1875 s at 160
    // BPM): it fills the silences instead of smearing them.
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
    // Triangle: the softest rung of the track's timbre ladder
    // (triangle -> sawtooth). The bass only bares its teeth
    // from TENSION onward.
    .sound("triangle")
    .decay(.3)
    .sustain(.2)
    // The bass keeps a short release: a bass tail would make
    // the switch muddy instead of smooth.
    .release(.12)
    .gain(.72)

// Drone of open fifths, way back in the mix: it states the
// grid without ever pulling the ear, and with no third it
// cannot accidentally sound major.
const droneStalk =
  note(`<
    [e3,b3] [e3,b3] [c3,g3] [d3,a3]
    [a2,e3] [c3,g3] [b2,f#3] [e3,b3]
  >`)
    .sound("triangle")
    // Soft attack: the drone swells in, it never clicks in,
    // including when it arrives through a switch.
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
// The robots are closing in. The bite goes to sixteenths and
// repeats without catching its breath: hammer, hammer, fall,
// riposte. The bass switches to sawtooth and alternates low
// octave / high octave across the eight eighths — the energy
// is carried by that, not by the volume.
// The lead line holds one single rest across the whole eight
// bars, at the very end of bar 8: the phrase takes its run-up
// there and nowhere else.
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
    // Shorter than movement 1 (the rate is doubled), but never
    // below ~0.15 s: under that the tail no longer covers the
    // switch and pick becomes audible as a cut.
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
    // Sawtooth: the timbre rung that separates observation from
    // combat. No distortion exists in the NES palette — the
    // grit comes from the waveform, a very short decay and the
    // gain, not from an effect.
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
// The bite is screamed an octave higher, the bass goes to
// sixteenths on the repeated note (12 attacks per bar against
// 8) and a second square voice hammers a syncopated 3+3+2
// pedal. Bar 7 is the one bar where that pedal touches the
// leading tone, echoing the lead's D sharp. Bar 7 is the only
// bar in the track where all three voices — lead, pedal and
// bass — spell that note out.
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
    // ENDGAME is the densest state in the track, so it is the
    // one that sets the ceiling for the whole mix. Measured on
    // an analyser tapped on the PATTERN BUS (a temporary
    // .analyze() on the final chain — upstream of superdough's
    // own output stage, so these are bus peaks, not speaker
    // peaks), volume slider at its maximum of 1:
    //     STALK 0.65   TENSION 0.90   ENDGAME 0.99
    // All six gains in this movement — lead, counter-voice,
    // bass, kick, roll, noise — were trimmed until that last
    // figure sat just under full scale. At their "natural"
    // values the stack peaked at 1.10 and clipped on the
    // downbeats. The ladder must stay in that order: ENDGAME
    // louder than TENSION louder than STALK.
    .gain(.55)

// Second voice: a syncopated 3+3+2 pedal, it tightens the
// rhythm against the bass instead of accompanying it.
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

    // Full roll every 4 bars: it pushes toward the end.
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
// Going from the stake out to the fight in one step was too
// brutal: we want to HEAR the machine spool up. The bridge
// lasts 6 bars (9 s) and accelerates exactly one thing, but it
// accelerates it everywhere at once: SUBDIVISION.
//
// Measured in the REPL on this exact file (onsets per bar,
// STALK and TENSION given for reference):
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
// Bar 6 deliberately overshoots TENSION on every line: it is
// the redline before the drop, not a preview of it.
//
// GRID: Em - Am - C - D - C - E5.
//
// WHY THE LAST BAR IS AN OPEN FIFTH (E/B, NO THIRD) and not a
// B7 dominant: the bridge is 6 bars long, the grid is 8. So
// TENSION resumes at bar (start + 6) mod 8 — i.e. any bar at
// all. A B7 would end on a D SHARP and could run straight into
// bar 4 (D chord, D NATURAL hammered by the bass): a cross
// relation, the leading tone falling instead of rising. An
// open E/B fifth has no such problem: it is consonant with all
// five chords of the grid. And a unison hammer into the fight
// reads more like "engine redlining" than a cadence anyway.
//
// EXITING INTO ENDGAME (0 -> 2) needs no new material, and it
// fits TIGHTER than the exit into TENSION. Checked line by
// line against movement 3:
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
// So into ENDGAME bar 6 mostly LANDS instead of overshooting —
// only the snare and the lead still redline, and they resolve
// downward by a single step. Register agrees too: bar 6 lives
// on E and B in octaves 5-6, which is exactly ENDGAME's range,
// where TENSION sits an octave lower. And bar 6 spells nothing
// but E and B, so it holds no D sharp: the cross relation
// above cannot bite whatever bar of the grid ENDGAME resumes
// on. The bass is already sawtooth on both sides.
//
// Each bridge bar is a ONE-CYCLE pattern (not a slowcat): we
// choose which one to play with `bridgeBar`. That is
// deliberate — a one-cycle pattern plays identically whatever
// the absolute cycle number, so the bridge always unfolds in
// the order 1, 2, 3, 4, 5, 6 wherever we are in the track.
//
// WRITING RULE FOR BRIDGE BARS: every bar must have an EVENT
// ON ITS DOWNBEAT, on every layer. See the REPL note in the
// decelerating bridge: on a step change, the head note of the
// PREVIOUS step is retriggered once. It only goes unnoticed if
// something lands at the same instant to mask it.
// ============================================================

const BRIDGE_UP_BARS = 6


// Switch memory. These five variables are the ONLY mutable
// state in the track: Strudel has no notion of "previous
// slider value", so we have to keep it ourselves.
//
// -Infinity = "bridge not armed". That is the neutral value,
// not Infinity: with -Infinity both bridge-progress tests
// (x < start, x - start < LENGTH) are false, so the requested
// state comes out IMMEDIATELY. With Infinity, x < start was
// always true and an arrival into TENSION that did not come
// from STALK (2 -> 1) stayed stuck on STALK forever. Bug
// fixed, tested in the REPL.
//
// THREE things depend on that sentinel now, not one:
//  - the instant switches, as above;
//  - the climb's `else e = mv` exit, which is what lets the
//    accelerating bridge hand off to ENDGAME as well as to
//    TENSION. With Infinity, 1 -> 2 would stick too, not just
//    2 -> 1;
//  - _bridgeFrom, which is read ONLY under
//    `x < _bridgeDownStart` — a branch unreachable while the
//    sentinel is -Infinity. That is why it needs no "nothing
//    pending" value of its own.
//
// _bridgeFrom remembers WHICH movement the decelerating bridge
// is leaving (1 or 2), so the fragment of a bar before the
// bridge's downbeat finishes in the outgoing movement. Before
// ENDGAME could reach that bridge this was always TENSION and
// the code simply said `e = 1`.
//
// At load time _prevState is 0: if the slider is already on 1
// OR ON 2 when the code is evaluated, the track therefore
// starts through the accelerating bridge rather than straight
// into TENSION / ENDGAME. That is intended — in game we always
// start in STALK, and in the REPL the spool-up is what we want
// to hear.
let _prevSlider = null
let _prevState = 0
let _bridgeUpStart = -Infinity
let _bridgeDownStart = -Infinity
let _bridgeFrom = 0


// Reads the current slider value. `movement` is a Pattern: we
// query it over a tiny span and take the value. It is read
// live, not frozen at the start of the cycle.
const readMovement = (t) => {
  const e = movement.queryArc(t, t + 1e-6)
  return e.length ? e[0].value : 0
}


// Bar index inside the accelerating bridge: 0..5.
const bridgeUpBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeUpStart)
  return Math.min(BRIDGE_UP_BARS - 1, Math.max(0, i))
})


// STALK's bite, bar after bar, eaten away by sixteenths until
// it becomes TENSION's hammer. The last bar plays nothing but
// E and B: the unison.
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


// The bass walks the same road: pulse, then eighths, then
// sixteenths. The timbre climbs with it — triangle for the
// first three bars, sawtooth from the fourth on, where the
// sixteenths settle. So .sound() is carried by EACH entry and
// specifically not applied after the pick, which would
// overwrite it.
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


// Builds the drum kit for one bridge bar. We keep .bank() on
// the 808 layer only: white noise is not part of that bank and
// a global .bank() would break it.
//
// `att` attenuates all three layers at once. It is 1 for the
// whole accelerating bridge (the acceleration alone is enough
// to raise the pressure) and decreases bar by bar in the
// decelerating bridge, so it lands on STALK's level with no
// step. The arithmetic is pinned to the real drum gains of the
// two movements: .72 * 1 ~= .75 (TENSION's kick) at the top of
// the ladder, .72 * .7 ~= .50 (STALK's kick) at the bottom.
// Changing STALK's or TENSION's drum gain means redoing this
// arithmetic, otherwise the last bridge bar makes a step.
// ENDGAME is reachable through both bridges now (0 -> 2 and
// 2 -> 0), but it needs no arithmetic of its own: its kick sits
// at .72, which is exactly `.72 * 1`, the att both bridges use
// at their loud end. So the climb lands on ENDGAME's level as
// cleanly as on TENSION's, and coming the other way the
// decelerating bridge's first bar is also .72. What you hear on
// a 2 -> 0 downbeat is therefore a DENSITY step (hats 16 -> 8)
// with no volume step, which is the point of the manoeuvre.
const bridgeDrums = (skins, hats, noise, att = 1) =>
  stack(
    s(skins).bank("RolandTR808").gain(.72 * att),
    s(hats).bank("RolandTR808").gain(.5 * att),
    s(noise).hpf(9000).decay(.02).gain(.26 * att)
  )

// This is where the acceleration is most plainly audible: the
// hats go from 2 to 16 hits per bar, the kick fills in, the
// snare arrives and then rolls.
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
// The mirror of the previous one. Falling out of TENSION
// (full hats, sawtooth bass, sixteenth-note melody) straight
// into STALK's stake out did not read as a release, it read as
// a breakdown: the climb existed, the descent did not, and the
// asymmetry was audible. So we wind back down the same thing
// we wound up: SUBDIVISION.
//
// Measured in the REPL on this exact file (onsets per bar):
//
//   bar       hats  kick  snare  noise  bass  lead  wave
//   TENSION     8     3     2      4      8    12   saw
//   bridge 1    8     3     2      4     12    12   saw
//   bridge 2    6     2     2      2      8     8   saw
//   bridge 3    4     2     1      1      6     5   TRI
//   bridge 4    2     2     1      1      4     4   tri
//   STALK       2     2     1     0/1     4     4   tri
//
// Bar 1 keeps TENSION's hats but pushes the BASS up to 12: the
// fall starts by leaning forward, so the first thing you hear
// is not a drop in energy but a change of gear. Bar 4 matches
// STALK line for line — only the open-fifth drone is missing,
// and it swells in on the next bar.
//
// ENTERING IT FROM ENDGAME (2 -> 0): the table above starts
// from TENSION, and that is what ENDGAME gets too. Bar 1's
// hats are 8 against ENDGAME's 16, so the bridge's downbeat
// carries an audible density step. That step IS the manoeuvre:
// 2 -> 0 is spelled "drop to fighting density, then wind
// down", the exact mirror of the climb's 0 -> 2 ("spool up to
// fighting density, then keep going"). No new bar was written
// for it, and nothing steps in VOLUME — bar 1's skins sit at
// .72 * 1 and ENDGAME's kick at .72.
//
// The bridge melody also descends in REGISTER bar after bar
// (octave 5-6 -> octave 4): the reverse of the accelerating
// bridge's road, and it makes the fall audible even if you are
// not listening to the hats.
//
// WHY 4 BARS AND NOT 6:
// a climb is something you wait for, a fall is something you
// feel. Six steps down would force repeated densities
// (8-6-6-4-4-2) and the track would drag exactly when it
// should be relaxing. Four bars = 6 s, one hat step per bar
// (8-6-4-2), every step audible, and the length lands exactly
// on half the 8-bar phrase.
//
// GRID: Am - C - D - Em, i.e. iv - VI - VII - i.
// The accelerating bridge ends on an open fifth because it can
// come out on any bar of the grid. Here it is the opposite: we
// want to be ALREADY settled when STALK resumes, so the
// cadence closes inside the bridge, on the E of bar 4. Since
// pick preserves phase, STALK can resume on any bar of its
// grid — and after an E minor, every chord of the grid (Em, C,
// D, Am, B) is a natural continuation.
//
// And bar 4 literally quotes bar 1 of STALK (melody E-E-B,
// bass E/B, "bd ~ bd ~ / hh ~ hh ~"): the bridge does not
// stop, it becomes the next movement.
// ============================================================

const BRIDGE_DOWN_BARS = 4


// Bar index inside the decelerating bridge: 0..3.
const bridgeDownBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeDownStart)
  return Math.min(BRIDGE_DOWN_BARS - 1, Math.max(0, i))
})


// TENSION's hammer un-syncopates itself: the sixteenths drop
// away one by one until nothing is left but the bare bite, and
// the line falls an octave and a half.
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
    // Slightly longer release than in the accelerating bridge:
    // here the tail is what builds the fade into STALK's stake
    // out, whereas on the way up it had to stay crisp.
    .release(.3)
    .room(.24)
    .roomsize(1.8)
    .gain(.54)


// The bass walks bassBridgeUp's road backwards: sixteenths,
// then eighths, then STALK's pulse — and the timbre comes back
// down with it, sawtooth -> triangle.
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


// The hats come down 8 -> 6 -> 4 -> 2, the kick empties out,
// the snare fades, and the attenuation follows the density
// drop so we land on STALK's level.
//
// Measured in the REPL: on a downbeat where the step changes,
// the PREVIOUS step's note is still triggered once, on top of
// the new one (pick samples the signal just before the bar
// line). That has always been true of the accelerating bridge,
// where it is inaudible since both hits land at the same
// instant with the same gain. Here the gains differ from step
// to step: the doubled hit lands on the downbeat, where the
// kick masks it. Nothing to fix — but it is the reason for the
// WRITING RULE above: a bridge bar starting on a rest would
// let the previous step bleed through in the open.
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
// THE STATE ACTUALLY PLAYING: 0, 1, 2 like the slider, plus 3
// ("accelerating bridge") and 4 ("decelerating bridge").
//
// signal(fn) gives a continuous pattern whose fn receives time
// in cycles — which is what lets us count the bars elapsed
// since the switch. Verified in the REPL: used as a pick
// selector, the signal is sampled at the start of each bar, so
// the bridge steps land exactly on downbeats.
//
// Math.ceil pins the bridge start to the NEXT BAR: we finish
// the current bar cleanly in the outgoing movement, then the
// bridge starts on a downbeat. Without it the bridge would
// start mid-bar and the steps would be offset.
//
// ARMING: a bridge only arms if the OUTGOING STATE suits it —
// STALK (0) for the accelerating one, TENSION or ENDGAME
// (1 or 2) for the decelerating one — and definitely not if
// the SLIDER reads those values, which is not the same thing.
// Between the slider gesture and Math.ceil there is a fragment
// of a bar during which the old movement is still sounding;
// testing the slider there would already see the new value. A
// fast 0 -> 1 -> 0 round trip inside that window would then arm
// a decelerating bridge out of a texture that never
// accelerated (and symmetrically). With _prevState, any round
// trip inside a bridge or its waiting window disarms both
// bridges and switches cleanly.
//
// DESTINATION-AGNOSTIC CLIMB: the accelerating bridge is
// "STALK spooling up", not "STALK becoming TENSION". So it
// arms for ANY non-zero slider value arriving from a played
// STALK, and its exit is `e = mv` — whatever the slider reads
// once the six bars are up. 0 -> 2 therefore gets the same ramp
// as 0 -> 1, and swapping the destination between 1 and 2 WHILE
// the climb runs simply RETARGETS the landing instead of
// aborting it. Verified in the REPL: the ramp keeps all six
// bars and lands on the new value.
//
// WHY THE RETARGET TEST READS THE BRIDGE CLOCK AND NOT
// _prevState: _prevState is rewritten on EVERY query, lookahead
// queries included, so at the instant of a slider gesture it
// may hold a state that is not audible yet. Testing it for "am
// I inside a bridge" (_prevState === 3) is precisely the family
// of ordering bug this file has already produced twice.
// `x - _bridgeUpStart < BRIDGE_UP_BARS` reads the bridge's own
// clock instead, and that only moves at arming time. The two
// _prevState tests that remain are safe because they look at
// SETTLED states (0, 1, 2), which hold steady across every
// query of a bar.
//
// Going back to a bridge's own start value still cuts it off
// dead, and it does so for a DIFFERENT reason on each side of
// the bridge's downbeat — which matters, because that is the
// distinction the original arming bug turned on:
//  - inside the waiting window (x < _bridgeUpStart), the climb
//    is not sounding yet and _prevState is still 0. mv === 0
//    then fails the `mv !== 0` guard on both arm branches, so
//    both bridges disarm and STALK simply continues;
//  - once the climb IS sounding, _prevState is 3. mv === 0
//    fails the retarget test, and 3 is neither 1 nor 2, so the
//    decelerating bridge cannot arm out of it either.
// Either way we never jump straight from one bridge into the
// other.
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
// This is where the smoothness of the transitions is decided.
//
// WHY NOT pickRestart ANYMORE:
// pickRestart restarts the chosen pattern on EVERY cycle.
// Verified in the REPL: on a phrase written as a slowcat it
// plays bar 1 forever (you get e4 e4 e4... instead of the
// whole phrase). An old version worked around that by cramming
// all 8 bars into a single 12 s cycle — which made movement
// switching take up to 12 s.
//
// pick restarts nothing: it just routes to the chosen pattern,
// which keeps running on ITS OWN clock. Two consequences, both
// verified in the REPL:
//
//  - Phase preserved. Switching on bar 5, the new movement
//    enters at ITS bar 5, not at the start of its phrase. Since
//    all three movements share the same chord grid, the harmony
//    continues with no seam: that is the real source of the
//    smoothness.
//
//  - Structure preserved. pick takes its structure from the
//    chosen pattern, not from the selector: the 8 eighths of
//    each bar come out intact.
//
// SWITCH LATENCY: slider() is read live, at the moment the
// scheduler queries the pattern — not frozen at the start of
// the cycle (verified: a mid-cycle query sees the new value
// right away). Instant switches therefore take effect on the
// NEXT SCHEDULED NOTE, i.e. one eighth, about 0.19 s at 160
// BPM.
//
// THE DELIBERATE EXCEPTIONS: every switch that leaves or
// enters the stake out goes through a bridge.
//   0 -> 1, 0 -> 2 : accelerating bridge, 6 bars (9 s). You
//            hear the change right away (at the latest on the
//            next bar, 1.5 s), but the switch takes ~10.5 s to
//            finish, the time it takes for the rhythm to really
//            accelerate. 0 -> 2 is that same climb: it just
//            keeps going into ENDGAME instead of settling into
//            TENSION.
//   1 -> 0, 2 -> 0 : decelerating bridge, 4 bars (6 s), ~7.5 s
//            total. Shorter on purpose: see the DECELERATING
//            BRIDGE section. From ENDGAME the wind-down opens
//            on TENSION's density (hats 16 -> 8 on the bridge's
//            downbeat), i.e. 2 -> 0 chains through 1.
// Only 1 <-> 2 stays instant, in both directions.
//
// Moving the slider during a bridge cuts it off dead, with one
// exception: swapping the DESTINATION of a running accelerating
// bridge (1 <-> 2) retargets it and keeps the whole ramp. Going
// back to the bridge's start value still aborts, and we never
// jump straight into the opposite bridge from inside a bridge.
//
// WHY IT DOES NOT CLICK: pick interrupts no sounding voice, it
// merely stops producing new ones. Notes already triggered
// finish their envelope normally. That is why the melodic
// layers keep a release of at least ~0.15 s plus a little
// reverb: their tail spills over onto the incoming movement
// and builds a real crossfade instead of a hard cut. The bass
// and the drums stay dry — a tail down low would make the
// switch muddy.
//
// Two syntax traps, also verified:
//
//  - Argument order. It really is selector.pick([...]).
//    Written the other way round (stack(...).pick(selector)),
//    Strudel goes into an infinite loop at evaluation.
//
//  - slider() returns a Pattern, not a number. Any raw
//    arithmetic on it (400 + filterOpenness * 2500) silently
//    yields NaN, then an AudioParam error on every note. So we
//    ALWAYS go through .range(min, max).
//
// Finally .mul(gain(volume)) and not .gain(volume): .gain()
// overwrites each layer's own gain (everyone ends up at the
// same level and the mix disappears), whereas .mul()
// multiplies it.
// ============================================================

// The array order IS the state encoding: 0 STALK, 1 TENSION,
// 2 ENDGAME, 3 accelerating bridge, 4 decelerating bridge.
// pick is positional and reports nothing if it is off by one:
// an entry inserted in the middle would silently play the
// wrong movement.
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
