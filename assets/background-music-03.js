// ============================================================
// TYLED
// Top-down 2D action / strategy
// ------------------------------------------------------------
// Third piece, same game, same NES palette (square, triangle,
// sawtooth, sine, 808, filtered white noise), same subject as
// -02 — a robot fight — but COLOSSAL. Where -02 is a stakeout
// turning into a knife fight (E minor, hammered repeated note,
// a jaw closing), this is two war machines the size of
// buildings walking at each other: C minor with a Neapolitan, a
// striding motif that leaps a fifth, a reactor arpeggio that
// spins up as the fight starts.
//
// NOT a re-skin of -02: different key, mode colour, motif, grid,
// movement concepts, plus the reactor arp layer with no
// counterpart there. DOES reuse -02's control mechanism —
// movement slider, two bridges, the signal()/pick() state
// machine — deliberately: proven, and the game drives all three
// files the same way.
// ------------------------------------------------------------
// MOVEMENT 1 : players far apart  -> the approach
// MOVEMENT 2 : players closing in -> contact, exchange of fire
// MOVEMENT 3 : endgame            -> reactor overload
//
// The "movement" control at top switches parts. Every switch
// leaving/entering the approach goes through a BRIDGE (see
// bottom of file):
//   0 -> 1, 0 -> 2 : ACCELERATING, 6 bars — machine spools up
//            (hats 2->16/bar, bass 4->12, reactor lights up at
//            bar 3, climbs 6->24). DESTINATION-AGNOSTIC:
//            "APPROACH spooling up", not "becoming ENGAGE" —
//            lands wherever the slider reads after six bars.
//   1 -> 0, 2 -> 0 : DECELERATING, 4 bars — winds down (hats
//            8->2, bass 12->4, reactor cuts out after one bar).
//            Starts one step below the climb's top since it
//            starts from ENGAGE's density (hats 8), not the
//            bridge's peak: 2->0 reads "drop to fighting
//            density, then wind down".
// Only 1 <-> 2 stays immediate: both already at fighting
// density, nothing to spool.
// ============================================================


// ============================================================
// CONTROL PANEL
// ============================================================

// 0 = APPROACH
// 1 = ENGAGE
// 2 = OVERLOAD

const movement = slider(0, 0, 2, 1)


// Master volume
const volume = slider(0.8, 0, 1, 0.01)


// Strategic filter: 0 = closed/muffled/far, 1 = open/bright/close.
// Acts on the WHOLE mix, drums included — the track has to be
// killable with a single gesture.
//
// So no layer sets its own .lpf() (would be overwritten).
// Timbral contrast comes from WAVEFORM (sine -> triangle ->
// square -> sawtooth) and envelopes instead. Two params survive
// since neither is the low-pass cutoff: white noise .hpf()
// (different filter), and the reactor arp's .fm() (shapes the
// oscillator, doesn't filter it).
const filterOpenness = slider(0.45, 0, 1, 0.01)


// ============================================================
// GLOBAL SETTINGS
//
// CAREFUL: setcpm counts CYCLES per minute, not quarter notes.
// ONE CYCLE = ONE BAR of four beats at 138 BPM: 138/4 = 34.5
// cycles/min, 1.739 s/bar.
//
// 138, not -02's 160, deliberately SLOWER: this piece has to
// sound heavy — same subdivision reads as mass on a long bar,
// speed on a short one. At 138 the sixteenth is 109 ms and the
// reactor's fastest rung (24/bar, see below) is 72 ms — still a
// shimmer, but kick and bass have room to sound big. -02 wanted
// the sixteenth to read as an engine, so it went faster instead.
//
// Phrase is always 8 bars, written as a slowcat < ... > (8 bars
// = 8 cycles) — natural Strudel spelling, and what makes
// movement switching fast (see bottom of file).
//
// So every melodic line is < [bar1] ... [bar8] >, 8 eighth-note
// slots/bar (a slot may split into sixteenths: [x y]). Drums are
// written over ONE bar and repeat themselves: NO .fast(8).
//
// THE .fast() TRAP, which bit this file while it was written:
// the NES reference (see THE 2A03 SPLIT) writes its triangle
// counter-melody as a one-bar string with .fast(2) — works there
// because the string is ONE cycle long. On an 8-bar slowcat,
// .fast(2) speeds up the SLOWCAT too, desyncing the grid from
// the drums. So the counter-melody here is written at its real
// subdivision instead, as pairs [x y] inside the eight slots.
// Never put .fast()/.slow() on anything carrying the 8-bar grid.
// ============================================================

setcpm(138 / 4)


// ============================================================
// THE MOTIF — "THE STRIDE"
//
// One three-note gesture drives the track:
//     C - G - Ab     (LEAP UP a fifth, then a semitone above it)
// Leap = footfall, semitone above = weight settling — opposite
// shape to -02's "bite" (repeated note, fall of a fourth): a
// fall is a jaw closing, a leap-plus-lean is something huge
// taking a step.
//
// The stride TRANSPOSES BY SHAPE (root, fifth, semitone above
// the fifth), not scale degree:
//     over Cm  ->  C  - G  - Ab   (Ab diatonic to C minor)
//     over Fm  ->  F  - C  - Db   (Db is the b2, NOT diatonic)
// So the stride itself drags a Db into the piece. Rather than
// fight it, the harmony adopts it — pitch collection is
//     C  Db  Eb  F  G  Ab  Bb     (C phrygian)
// plus one borrowing, bar 7's B natural (see grid). Phrygian, not
// plain minor, is why this sounds monumental — the b2 leaning on
// the tonic.
//
// The shape is kept literally only where its third note stays in
// that collection (Cm, Fm bars). Over Ab it would produce
// Ab-Eb-E natural, which belongs nowhere here, so those bars
// fall back to a chord tone instead.
//
// All three movements share tempo, key and 8-bar grid:
//     Cm - Cm - Ab - Db - Fm - Ab - G  - Cm
//      i    i   VI   bII  iv   VI   V    i
// This is what makes switches seamless: bar 5 of ENGAGE
// harmonically continues bar 4 of APPROACH.
//
// THE Db / D-NATURAL FIREWALL.
// The grid has a Db major (bar 4, the Neapolitan, the piece's
// one huge chord) and a G major (bar 7, borrowed from harmonic
// minor, the only B/D natural in the track). Db and D natural a
// semitone apart is a cross relation — sounds like a mistake, not
// a colour. So one rule holds everywhere, movements and bridges
// alike: D natural exists ONLY on bar 7; Db is free on bars 1-5
// and 8; bar 6 (Ab: Ab C Eb, neither D) is the AIRLOCK between
// them.
//
// So nothing added to bar 6 may spell either D. Same reason the
// accelerating bridge's grid has no D natural at all (see that
// section), and the decelerating bridge keeps its one Bb chord
// two bars from its one Db.
// ============================================================


// ============================================================
// THE 2A03 SPLIT  (how movement 1 is voiced)
//
// The NES chip has five non-interchangeable channels: two PULSE
// (square), one TRIANGLE, one NOISE, one DMC sample channel.
// Real NES soundtracks write to that constraint — one pulse
// takes bass, the other lead, triangle plays a fast counter-line
// above both, noise is the kit.
//
// APPROACH is voiced to that layout (thin enough for channels to
// read as channels). Reference: a public NES-style Strudel
// sketch (title_screen.strudel, ryanfoxeth/strudel-cli), channel
// assignment adopted in principle, not material:
//
//     pulse 1   square    bass line, heavily rested
//     pulse 2   square    the stride, the actual tune
//     triangle  triangle  fast counter-melody, high
//     "DMC"     sine      soft offbeat accents
//     noise     808 + filtered white noise
//
// The sine extends -02's palette (square/triangle/sawtooth/
// 808/noise) deliberately: stands in for the DMC channel (on
// hardware, a low-rate sample player for soft percussive
// accents) — a sine at very low gain is the closest thing to "a
// pad that isn't really a pad". APPROACH only; once the fight
// starts it's replaced by the reactor arp — the machines stop
// breathing and start running.
//
// Movements 2 and 3 abandon the split on purpose: in the fight
// the channels merge into one mass, which is the point.
// ============================================================


// ============================================================
// THE REACTOR ARP  (how movements 2 and 3 get their shimmer)
//
// One layer, shared by ENGAGE, OVERLOAD and the top of both
// bridges — the piece's dramaturgical spine, the machines'
// reactor. SILENT in APPROACH, lights up bar 3 of the
// accelerating bridge, climbs to peak as the bridge redlines,
// runs half speed through ENGAGE, full speed through OVERLOAD,
// cuts out after one bar of the decelerating bridge. You can
// tell the movement with your eyes shut from this layer alone.
//
// It LOOKS like a live arpeggio (community sketch "8 bit arp
// example", boggo x osc()_peterson: scale-degree chords, .arp()
// at a patterned speed, periodic .rev(), .fm() for pulse-width
// movement, a saw ramp as amplitude envelope) but isn't anymore
// — the live .arp() version is a confirmed Strudel bug (see WHY
// THIS IS WRITTEN AS LITERAL NOTES below), so every bar, every
// movement and bridge, is now a precomputed literal note()
// pattern.
//
// WHY PER-BAR .scale() ROOTS AND NOT ONE MODE NAME. One
// .scale("C4:phrygian") with degree-picked chords would be
// simpler, but: mode names beyond major/minor aren't documented
// in this repo's Strudel reference (silent failure risk), and a
// per-bar root spells each grid chord EXACTLY — including a real
// G major on bar 7, which a fixed C-rooted mode can't produce.
// Cost: degree numbers mean something different each bar, hence
// the table below.
//
// THE CHORD TABLE. Base voicing is degrees 0,2,4 (plain triad on
// the bar's root); three bars override via .when(<8-bar mask>,
// x => x.n(...)):
//
//   bar  chord  scale root   degrees   sounds
//    1   Cm     C4:minor     0,2,4     C4  Eb4 G4
//    2   Cm     C4:minor     0,2,4     C4  Eb4 G4
//    3   Ab     Ab3:major    2,4,6     C4  Eb4 G4   (Abmaj7, rootless)
//    4   Db     Db4:major    2,4,7     F4  Ab4 Db5  (first inversion, lifted)
//    5   Fm     F4:minor     0,2,4     F4  Ab4 C5
//    6   Ab     Ab3:major    2,4,6     C4  Eb4 G4   (Abmaj7, rootless)
//    7   G      G3:major     0,4,7     G3  D4  G4   (OPEN FIFTH, no third)
//    8   Cm     C4:minor     0,2,4     C4  Eb4 G4
//
// Three choices behind it:
//  - bar 7 thirdless: the B natural is the track's one borrowed
//    note, belongs to the LEAD; doubled at 24 notes/bar it'd
//    become wallpaper. Open G/D/G lets it land alone.
//  - bar 4 inverted upward (F Ab Db5), not rooted: the bass
//    already hammers Db an octave and a half below; doubling the
//    root made the Neapolitan sound thick, not tall.
//  - bars 3 and 6 drop the Ab root, play C Eb G: keeps the
//    airlock clean (no D of either kind) and are the only bars
//    where the arp's pitch content doesn't move.
//
// WHY BARS 3 AND 7 RUN BACKWARDS. Formerly a `.when("<0 0 1 0>",
// x => x.rev())` mask keyed to absolute cycle — since 4 divides
// 8, cycle mod 4 == 2 always meant grid bars 3 and 7. Now baked
// in by hand: those bars spell their three-note group back to
// front in the note() string, forward everywhere else. Same
// accent, no live .rev().
//
// ARP SPEED IS IN GROUPS, NOT NOTES. Built on .arp("[0 1 2]*N")
// — N repeats of a three-note group/cycle, onset count 3N. Each
// literal bar below is "[n1 n2 n3]*N", that call's precomputed
// output:
//     N = 2   ->  6 notes/bar   bridge bar 3, the reactor lights
//     N = 4   ->  12            ENGAGE, and bridge bar 4
//     N = 6   ->  18            bridge bar 5
//     N = 8   ->  24            OVERLOAD, and bridge bar 6
// The reference sketch uses <12 16 20> (36-60 notes/cycle, 29-48
// ms/note at 138 BPM) — past reading as pitch, and it fought the
// kick. 24/bar (72 ms) is the top of what stays musical here.
//
// .fm() ON A SQUARE is the pulse-width analogue: the 2A03's
// pulse channels sweep duty-cycle registers for THE recognisable
// NES lead sound; Strudel has no duty parameter, but FM on a
// square carrier moves harmonic content comparably, and survives
// filterOpenness (unlike a filter sweep). Each movement gets its
// own fm slowcat so timbre thickens with the stakes.
//
// THE SAW AMPLITUDE ENVELOPE. .gain(saw.range(peak, 0)) ramps
// peak to silence across one cycle — a capacitor discharging once
// per bar. Phrase-level, not note-level: reactor charges on the
// downbeat, bleeds out by the bar line, pinning the arp to the
// grid instead of washing. OVERLOAD halves the period (.slow(.5),
// two discharges/bar) — twice as frantic even where note count
// is unchanged. Note: saw.range(peak, 0) really does go peak->0;
// .range() doesn't sort its arguments.
// ============================================================

// WHY THIS IS WRITTEN AS LITERAL NOTES, NOT A LIVE .arp() CALL.
//
// Used to be built dynamically: per-bar .scale() root, .when()
// chord overrides (see THE CHORD TABLE above), fed through
// chords.arp('[0 1 2]*N') and an occasional .when(mask, rev).
//
// That's a CONFIRMED Strudel bug when the result sits inside a
// signal()-driven pick() — where every layer in this file lives.
// Root-caused this session on a cache-cleared bundle:
//   1. -02 (no .arp() anywhere) survived dense slider-drag stress
//      on every transition with zero "[query] error" hits;
//   2. -03 with dynamic chords.arp() threw that exact error
//      reliably under the same stress, on transitions touching
//      ENGAGE/OVERLOAD;
//   3. removing only the .arp() call (chords.arp(...) -> chords)
//      made the error disappear;
//   4. precomputed literal note() patterns (this factory), also
//      clean.
// Conclusion: a chords.arp(...) Pattern in a signal()-driven
// pick() is unsafe under slider drag; a plain note() pattern is
// fine — same conclusion as the bridges' silent-bar bug below
// (arpBridgeUp/arpBridgeDown). This factory is now the ONE
// reactor-voice implementation, no live .arp()/.rev() left
// anywhere in the file.
//
//   notes      literal, already-arpeggiated note() string, e.g.
//              "[c4 eb4 g4]*4" = precomputed chords.arp('[0 1 2]*4')
//              on a Cm triad. A backwards bar (WHY BARS 3 AND 7
//              RUN BACKWARDS) just spells its group in reverse.
//   peak       top of the saw envelope (the layer's gain)
//   fm         modulation index, a pattern string
//   envBars    length of one saw discharge in bars
const reactorVoiceLiteral = (notes, peak, fm, envBars = 1) =>
  note(notes)
    .sound("square")
    .fm(fm)
    .decay(.07)
    .sustain(0)
    .release(.05)
    .room(.2)
    .roomsize(2)
    .gain(saw.range(peak, 0).slow(envBars))

// ============================================================
// GAIN STAGING — READ THIS BEFORE CHANGING ANY .gain()
//
// -02's measured ceiling (volume slider at 1, bus peaks via a
// temporary .analyze() on the final chain): STALK 0.65, TENSION
// 0.90, ENDGAME 0.99 — its densest movement trimmed just under
// full scale.
//
// Numbers in THIS file are BUDGETED from that, not independently
// measured yet — confirm via .analyze() before treating as final.
//
// One non-obvious move: every layer with a -02 counterpart sits
// ~10-15% BELOW it. Arithmetic, not timidity — OVERLOAD carries
// SEVEN layers plus the reactor arp, where -02's ENDGAME carried
// six and was already at 0.99; copying -02's gains and adding an
// eighth layer reproduces the clipping -02 had to fix.
//
// Ladder must stay in this order (and match the bridge arithmetic
// further down):
//     APPROACH quieter than ENGAGE quieter than OVERLOAD
//     APPROACH skins .48   hats .34
//     ENGAGE   skins .66   hats .46
//     OVERLOAD skins .66   hats .46   (density, not volume,
//                                      separates it)
// ============================================================


// ============================================================
// MOVEMENT 1 : APPROACH
// Two machines walking toward each other across open ground.
// Strict 2A03 channel split (see THE 2A03 SPLIT): pulse 1 rested
// bass, pulse 2 states the stride, triangle runs a fast
// counter-line two octaves above, sine breathes on the offbeats,
// kit is two footfalls and a hat.
// Grid: Cm - Cm - Ab - Db - Fm - Ab - G - Cm.
// ============================================================

// PULSE 2 — the stride, bare, four onsets/bar. Bar 1 is the motif
// in reference form: C-G-Ab-G. Bar 5 transposes it by shape over
// Fm, where Db enters the piece. Bar 7 approaches the borrowed B
// natural from Eb above (harmonic-minor gesture) so the leading
// tone arrives as a resolution, not an accident.
const melodyApproach =
  note(`<
    [c4  ~   g4  ~   ab4 ~   g4  ~  ]
    [~   c4  ~   g4  ~   ab4 g4  eb4]
    [ab4 ~   eb5 ~   c5  ~   ab4 ~  ]
    [~   db5 ~   ab4 ~   f4  ab4 ~  ]
    [f4  ~   c5  ~   db5 ~   c5  ~  ]
    [~   ab4 ~   eb5 ~   c5  eb5 ~  ]
    [g4  ~   d5  ~   eb5 d5  b4  ~  ]
    [~   c5  ~   g4  ~   eb4 c4  ~  ]
  >`)
    .sound("square")
    .decay(.14)
    .sustain(.22)
    // Long release + reverb/echo: tail turns a movement switch
    // into a fade (see transitions note at bottom). Echo tuned to
    // the eighth note at 138 BPM (0.2174 s): fills silences
    // between strides instead of smearing them.
    .release(.32)
    .room(.28)
    .roomsize(2)
    .delay(.24)
    .delaytime(.2174)
    .delayfeedback(.3)
    .gain(.46)

// PULSE 1 — bass, square not triangle (triangle's spoken for by
// the counter-melody); a rested square bass with a hard attack
// is a foot hitting ground. Four onsets/bar, always
// root-root-fifth-root: a gait, not a walking line.
const bassApproach =
  note(`<
    [c2  ~ c2  ~ g1  ~ c2  ~]
    [c2  ~ c2  ~ eb2 ~ g1  ~]
    [ab1 ~ ab1 ~ eb2 ~ ab1 ~]
    [db2 ~ db2 ~ ab1 ~ db2 ~]
    [f1  ~ f1  ~ c2  ~ f1  ~]
    [ab1 ~ ab1 ~ eb2 ~ c2  ~]
    [g1  ~ g1  ~ d2  ~ b1  ~]
    [c2  ~ c2  ~ g1  ~ c2  ~]
  >`)
    .sound("square")
    .decay(.22)
    .sustain(.14)
    // Short release: a bass tail would muddy the switch and
    // blur the gait.
    .release(.1)
    .gain(.4)

// TRIANGLE — fast counter-melody, written as sixteenth PAIRS
// instead of .fast(2) (see THE .fast() TRAP). Eight onsets/bar,
// two octaves above the bass, always on the beat — interlocks
// with the offbeat-only sine below.
const counterApproach =
  note(`<
    [[g5  eb5] ~ [g5  eb5] ~ [c6  g5 ] ~ [eb5 c5 ] ~]
    [[g5  eb5] ~ [c6  g5 ] ~ [eb5 c5 ] ~ [g5  eb5] ~]
    [[eb6 c6 ] ~ [ab5 eb5] ~ [c6  ab5] ~ [eb6 c6 ] ~]
    [[f5  db5] ~ [ab5 f5 ] ~ [db6 ab5] ~ [f5  db5] ~]
    [[c6  ab5] ~ [f5  c5 ] ~ [ab5 f5 ] ~ [c6  ab5] ~]
    [[eb6 c6 ] ~ [ab5 eb5] ~ [c6  ab5] ~ [g5  eb5] ~]
    [[d6  b5 ] ~ [g5  d5 ] ~ [b5  g5 ] ~ [d6  b5 ] ~]
    [[g5  eb5] ~ [c6  g5 ] ~ [eb5 c5 ] ~ [c5  ~  ] ~]
  >`)
    .sound("triangle")
    .decay(.08)
    .sustain(.05)
    .release(.09)
    .room(.3)
    .roomsize(2)
    // Quiet on purpose: on hardware the triangle has no volume
    // control, sits under the pulses by construction. Louder
    // makes the movement sound busy — what approach must not be.
    .gain(.16)

// "DMC" — sine, offbeats only. The movement's only sustained
// sound, and the only thing that gives APPROACH air.
const accentApproach =
  note(`<
    [~ c5  ~ c5  ~ g4  ~ c5 ]
    [~ c5  ~ g4  ~ c5  ~ eb5]
    [~ ab4 ~ ab4 ~ eb5 ~ ab4]
    [~ db5 ~ ab4 ~ db5 ~ f4 ]
    [~ f4  ~ c5  ~ f4  ~ ab4]
    [~ ab4 ~ eb5 ~ ab4 ~ c5 ]
    [~ g4  ~ d5  ~ g4  ~ b4 ]
    [~ c5  ~ g4  ~ c5  ~ ~  ]
  >`)
    .sound("sine")
    // Slow attack, long release: swells in, never clicks, even
    // via a switch.
    .attack(.2)
    .decay(.5)
    .sustain(.35)
    .release(.6)
    .room(.4)
    .roomsize(3)
    .gain(.12)

// NOISE CHANNEL. Kick on 1 and the last eighth — two unevenly
// spaced footfalls, heavy rather than metronomic. Hats split
// into their own layer/gain so the decelerating bridge's last
// bar can land exactly here (see bridgeDrums).
const drumsApproach =
  stack(
    s("bd ~ ~ bd, ~ ~ sd ~")
      .bank("RolandTR808")
      .gain(.48),

    s("hh ~ hh ~")
      .bank("RolandTR808")
      .gain(.34),

    // One low tom every four bars: the far machine's footfall
    // answering ours — the only hint of a second robot at all.
    s("<[lt ~ ~ ~ ~ ~ ~ ~] ~ ~ ~>")
      .bank("RolandTR808")
      .gain(.34),

    // A breath of noise every other bar, just to inhale.
    s("<~ [~ ~ ~ ~ ~ ~ ~ white]>")
      .hpf(9000)
      .decay(.02)
      .gain(.18)
  )

const movementApproach =
  stack(
    melodyApproach,
    bassApproach,
    counterApproach,
    accentApproach,
    drumsApproach
  )


// ============================================================
// MOVEMENT 2 : ENGAGE
// Contact. 2A03 split abandoned: both pulses collapse into one
// hammered lead an octave up, bass goes sawtooth and doubles to
// eight onsets/bar, reactor arp lights up at half speed
// (12 notes/bar). Triangle and sine gone — nothing breathes.
//
// Lead keeps the stride's LEAP inside every bar (slot 1 repeated
// root, slot 2 the fifth above) so the motif stays legible at
// four times the density.
// ============================================================

const melodyEngage =
  note(`<
    [[c5  c5 ] g5  [ab5 g5 ] c5  [eb5 c5 ] g4  [c5  eb5] g5 ]
    [[c5  c5 ] eb5 [g5  ab5] g5  [c6  bb5] g5  [eb5 g5 ] c5 ]
    [[ab5 ab5] eb5 [c6  ab5] eb5 [g5  ab5] c6  [eb6 c6 ] ab5]
    [[db6 db6] ab5 [f5  db5] ab5 [db6 f6 ] ab5 [f5  ab5] db6]
    [[f5  f5 ] c6  [ab5 f5 ] c5  [db6 c6 ] ab5 [f5  ab5] c6 ]
    [[ab5 ab5] eb6 [c6  ab5] eb5 [ab5 c6 ] eb6 [c6  ab5] eb5]
    [[g5  g5 ] d6  [b5  g5 ] d5  [eb6 d6 ] b5  [g5  b5 ] d6 ]
    [[c6  c6 ] g5  [eb5 c5 ] g5  [c6  bb5] g5  [ab5 g5 ] c5 ]
  >`)
    .sound("square")
    .decay(.08)
    .sustain(.12)
    // Shorter than APPROACH (rate trebled), but never below
    // ~0.15 s or the tail stops covering the switch and pick
    // becomes an audible cut.
    .release(.18)
    .room(.15)
    .roomsize(1.5)
    .gain(.52)

const bassEngage =
  note(`<
    [c1  c2  c1  c2  c1  c2  g1  bb1]
    [c1  c2  c1  c2  g1  c2  eb2 g2 ]
    [ab1 ab2 ab1 ab2 eb2 ab1 c2  eb2]
    [db2 db3 db2 db3 ab1 db2 f2  ab2]
    [f1  f2  f1  f2  c2  f1  ab1 c2 ]
    [ab1 ab2 ab1 ab2 eb2 ab1 eb2 g2 ]
    [g1  g2  g1  g2  d2  g1  b1  d2 ]
    [c1  c2  c1  c2  g1  c2  bb1 ab1]
  >`)
    // Sawtooth: the timbre rung separating approach from fight.
    // No distortion in this palette — grit comes from waveform,
    // short decay, and gain instead.
    .sound("sawtooth")
    .decay(.14)
    .sustain(.08)
    .release(.05)
    .gain(.66)

// Reactor at half speed, fm low and mostly steady: the machine is
// running, not straining. 12 notes/bar (N = 4, see ARP SPEED IS
// IN GROUPS); bars 3 and 7 backwards per WHY BARS 3 AND 7 RUN
// BACKWARDS.
const arpEngage = reactorVoiceLiteral(
  `<
    [c4 eb4 g4]*4
    [c4 eb4 g4]*4
    [g4 eb4 c4]*4
    [f4 ab4 db5]*4
    [f4 ab4 c5]*4
    [c4 eb4 g4]*4
    [g4 d4 g3]*4
    [c4 eb4 g4]*4
  >`,
  .22, "<.15 .4 .15 .8>", 1
)

const drumsEngage =
  stack(
    s("bd ~ bd bd, ~ sd ~ sd")
      .bank("RolandTR808")
      .gain(.66),

    s("hh*8")
      .bank("RolandTR808")
      .gain(.46),

    // Snare pickup on the 4th bar of each four. One cycle = one
    // bar, so this 4-element slowcat lands every 4 bars (bars 4
    // and 8 of the phrase).
    s("<~!3 [~ ~ sd sd]>")
      .bank("RolandTR808")
      .gain(.46),

    s("white*4")
      .hpf(9000)
      .decay(.02)
      .gain(.22)
  )

const movementEngage =
  stack(melodyEngage, bassEngage, arpEngage, drumsEngage)


// ============================================================
// MOVEMENT 3 : OVERLOAD
// The reactor past its limit. Lead screams an octave higher,
// bass hits twelve onsets/bar on a hammered root, a second
// square voice drives a syncopated 3+3+2 pedal against it, hats
// double to sixteen, arp doubles note count (24/bar) and
// envelope rate (two discharges/bar).
//
// Bar 7: lead, pedal and bass spell the leading tone together —
// the arp deliberately does NOT join them (see THE REACTOR ARP).
// Three voices on a note and one pointedly off it is louder than
// four voices on it.
// ============================================================

const melodyOverload =
  note(`<
    [[c6  c6 ] g5  [c6  c6 ] g5  [ab5 g5 ] eb5 [g5  c6 ] eb6]
    [[c6  c6 ] eb6 [g5  c6 ] g5  [bb5 c6 ] eb6 [g6  eb6] c6 ]
    [[ab5 ab5] eb6 [c6  ab5] eb5 [ab5 c6 ] eb6 [ab6 eb6] c6 ]
    [[db6 db6] ab5 [db6 f6 ] ab6 [f6  db6] ab5 [db6 f6 ] ab6]
    [[f5  f5 ] c6  [f6  c6 ] ab5 [c6  f6 ] ab6 [f6  c6 ] ab5]
    [[c6  c6 ] ab5 [eb6 c6 ] ab6 [eb6 c6 ] ab5 [c6  eb6] ab6]
    [[g5  g5 ] d6  [b5  g5 ] d5  [eb6 d6 ] b5  [d6  g6 ] b5 ]
    [[c6  c6 ] g5  [eb6 c6 ] g5  [c6  eb6] g6  [eb6 c6 ] g5 ]
  >`)
    .sound("square")
    .decay(.06)
    .sustain(.1)
    .release(.16)
    .room(.14)
    .roomsize(1.5)
    .gain(.46)

// Second voice: syncopated 3+3+2 pedal, tightens the rhythm
// AGAINST the bass — three eighths, three eighths, two eighths,
// accents landing where the kick does not.
const counterOverload =
  note(`<
    [c4  ~ ~ c4  ~ ~ g4  ~]
    [c4  ~ ~ c4  ~ ~ eb4 ~]
    [ab3 ~ ~ ab3 ~ ~ eb4 ~]
    [db4 ~ ~ db4 ~ ~ ab4 ~]
    [f3  ~ ~ f3  ~ ~ c4  ~]
    [ab3 ~ ~ ab3 ~ ~ c4  ~]
    [g3  ~ ~ g3  ~ ~ b3  ~]
    [c4  ~ ~ g3  ~ ~ c4  ~]
  >`)
    .sound("square")
    .decay(.05)
    .sustain(0)
    .release(.13)
    .room(.12)
    .gain(.2)

const bassOverload =
  note(`<
    [[c1  c1 ] c1  [c1  c2 ] c1  [c1  c1 ] c2  [g1  c2 ] c1 ]
    [[c1  c1 ] c1  [c1  c2 ] c1  [c1  c1 ] c2  [bb1 c2 ] eb2]
    [[ab1 ab1] ab1 [ab1 ab2] ab1 [ab1 ab1] ab2 [eb2 ab2] ab1]
    [[db2 db2] db2 [db2 db3] db2 [db2 db2] db3 [ab1 db3] db2]
    [[f1  f1 ] f1  [f1  f2 ] f1  [f1  f1 ] f2  [c2  f2 ] f1 ]
    [[ab1 ab1] ab1 [ab1 ab2] ab1 [ab1 ab1] ab2 [c2  eb2] g2 ]
    [[g1  g1 ] g1  [g1  g2 ] g1  [g1  g1 ] g2  [d2  g2 ] b1 ]
    [[c1  c1 ] c1  [c1  c2 ] c1  [c1  c1 ] c2  [g1  bb1] ab1]
  >`)
    .sound("sawtooth")
    .decay(.1)
    .sustain(.06)
    .release(.04)
    .gain(.56)

// Reactor at full speed, discharging twice a bar, fm pushed into
// unstable territory on the accents — the loudest cue the track
// has reached its last movement. 24 notes/bar (N = 8); bars 3
// and 7 backwards per WHY BARS 3 AND 7 RUN BACKWARDS.
const arpOverload = reactorVoiceLiteral(
  `<
    [c4 eb4 g4]*8
    [c4 eb4 g4]*8
    [g4 eb4 c4]*8
    [f4 ab4 db5]*8
    [f4 ab4 c5]*8
    [c4 eb4 g4]*8
    [g4 d4 g3]*8
    [c4 eb4 g4]*8
  >`,
  .26, "<.5 1.5 .5 2>", .5
)

const drumsOverload =
  stack(
    s("bd bd bd bd, ~ sd ~ sd")
      .bank("RolandTR808")
      .gain(.66),

    s("hh*16")
      .bank("RolandTR808")
      .gain(.46),

    // Full roll every 4 bars, pushing toward the end.
    s("<~!3 [sd sd sd sd]>")
      .bank("RolandTR808")
      .gain(.36),

    s("white*8")
      .hpf(9000)
      .decay(.015)
      .gain(.19)
  )

const movementOverload =
  stack(
    melodyOverload,
    counterOverload,
    bassOverload,
    arpOverload,
    drumsOverload
  )


// ============================================================
// ACCELERATING BRIDGE  (APPROACH -> ENGAGE / OVERLOAD)
//
// A straight cut from approach to fight was too brutal; we want
// to HEAR the machine spool up. 6 bars (10.4 s at 138 BPM),
// accelerating one thing everywhere: SUBDIVISION.
//
// Onsets per bar (APPROACH/ENGAGE/OVERLOAD for reference;
// APPROACH's lead reads 5 on bars 2 and 7 where the stride picks
// up a passing note — table quotes the common value 4):
//
//   bar       hats kick snare noise bass lead  arp  wave
//   APPROACH    2    2    1     0/1    4    4    0  sq+tri
//   bridge 1    2    2    0      0     4    4    6* tri
//   bridge 2    2    2    0      1     4    6    6* tri
//   bridge 3    4    2    1      2     6    8    6  tri
//   bridge 4    6    2    2      2     8   10   12  SAW
//   bridge 5    8    3    2      4     8   12   18  saw
//   bridge 6   16    4    4      8    12   16   24  saw
//   ENGAGE      8    3    2      4     8   12   12  saw
//   OVERLOAD   16    4    2/4    8    12   12   24  saw
//
// * bars 1-2's arp onsets are SILENT (peak = 0, see arpBridgeUp):
//   real triggers at zero gain, so the reactor is genuinely quiet
//   for two bars before igniting at bar 3 — listed as an onset
//   count, not a loudness count.
//
// Bar 6 deliberately overshoots ENGAGE — redline before the drop,
// not a preview of it.
//
// GRID: Cm - Fm - Ab - Db - Ab - C5.
//
// NO D NATURAL ANYWHERE IN THE BRIDGE, so the firewall (see THE
// MOTIF) can't be violated no matter which bar it exits onto: Cm,
// Fm, Ab contain neither D, and bar 4's Db is followed by Ab, not
// Bb. Bar 4 is the bridge's one huge chord, two thirds of the way
// up — committed but not yet redlining.
//
// LAST BAR IS AN OPEN FIFTH (C/G, no third), not a G7 dominant:
// bridge is 6 bars, grid is 8, so the movement resumes at bar
// (start+6) mod 8 — any bar. G7 ends on B NATURAL, risking a
// cross relation with bar 4's Db bass. Open C/G is consonant with
// all five chords here, no D of either spelling, and reads as
// "reactor redlining" rather than a cadence.
//
// EXITING INTO OVERLOAD (0 -> 2) needs no new material, fits
// TIGHTER than the ENGAGE exit:
//
//   line         bridge 6   OVERLOAD
//   hats            16        16
//   kick             4         4
//   snare            4         2  (4 on the roll bar)
//   noise            8         8
//   bass            12        12
//   arp             24        24
//   lead            16        12
//   skins gain    .66       .66
//
// Bar 6 mostly LANDS into OVERLOAD rather than overshooting —
// only snare and lead still redline. Register agrees (bar 6 sits
// in OVERLOAD's octaves 5-6). Bass already sawtooth both sides,
// arp already at top speed.
//
// Each bridge bar is a ONE-CYCLE pattern (not a slowcat), picked
// via `bridgeUpBar`: plays identically at any absolute cycle
// number, so the bridge always unfolds 1-6 wherever it starts.
// Same reason the reactor layer never gets a live .rev() keyed to
// absolute cycle (see WHY BARS 3 AND 7 RUN BACKWARDS) — that
// would make the bridge sound different depending on when it
// started.
//
// WRITING RULE: every bar needs an EVENT ON ITS DOWNBEAT, on
// every layer — a step change retriggers the previous step's
// head note once (see decelerating bridge), inaudible only if
// something masks it at the same instant. The arp gets this for
// free (a group always fires on beat 0); melody, bass and kit
// don't, so check by eye when rewriting a bar.
// ============================================================

const BRIDGE_UP_BARS = 6


// Switch memory: the ONLY mutable state in the track (Strudel has
// no "previous slider value" of its own).
//
// -Infinity = "bridge not armed" — not Infinity. With -Infinity
// both progress tests (x < start, x - start < LENGTH) read false,
// so the requested state applies IMMEDIATELY. With Infinity,
// x < start was always true and an arrival into ENGAGE not from
// APPROACH (2 -> 1) stuck on APPROACH forever — a real bug found
// and fixed in -02; sentinel carried over unchanged.
//
// Three things depend on that sentinel:
//  - the instant switches above;
//  - the climb's `else e = mv` exit, letting the accelerating
//    bridge hand off to OVERLOAD too, not just ENGAGE;
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
// straight into ENGAGE/OVERLOAD — intended, since the game always
// starts in APPROACH and the REPL wants to hear the spool-up.
let _prevSlider = null
let _prevState = 0
let _bridgeUpStart = -Infinity
let _bridgeDownStart = -Infinity
let _bridgeFrom = 0

// Last value readMovement actually read — the fallback on a
// failed read (see readMovement), closer to correct than
// hardcoding 0 since a bad read mid-drag is more likely near the
// slider's current position than at its start.
let _lastMovementValue = 0


// Reads the current slider value: `movement` queried over a tiny
// span, live, not frozen at cycle start.
//
// The slider widget is flaky mid-drag in two ways, both seen in
// the REPL: queryArc can return `[undefined]` (length truthy,
// element empty), or THROW outright from inside its own hap
// generation. filter(Boolean) handles the first; try/catch the
// second (a value never returned can't be filtered) — naive
// `e.length ? e[0].value : 0` catches neither, surfacing as
// "Cannot read properties of undefined (reading 'value')". On any
// failure, fall back to the last good value rather than 0, so a
// bad tick mid-drag doesn't yank the state machine back to
// APPROACH.
const readMovement = (t) => {
  try {
    const e = movement.queryArc(t, t + 1e-6).filter(Boolean)
    const v = e.length ? e[0].value : _lastMovementValue
    _lastMovementValue = v
    return v
  } catch {
    return _lastMovementValue
  }
}


// Bar index inside the accelerating bridge: 0..5.
const bridgeUpBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeUpStart)
  return Math.min(BRIDGE_UP_BARS - 1, Math.max(0, i))
})


// The stride eaten away by subdivision until it becomes ENGAGE's
// hammer. Register climbs an octave and a half (c5 -> c6),
// audible even without counting hats. Last bar: C and G, the
// unison.
const melodyBridgeUp =
  bridgeUpBar.pick([
    note("[c5  ~        g4  ~        ab4 ~        g4       ~  ]"),
    note("[f4  f4       ~   c5       [db5 c5]  ~  ab4      ~  ]"),
    note("[ab4 ab4      ~   eb5      [c5 bb4] ab4 [g4 ab4] ~  ]"),
    note("[db5 [db5 eb5] f5 [eb5 db5] ab5 [f5 g5] ab5      ~  ]"),
    note("[[eb5 f5] g5 [f5 eb5] c6 [ab5 bb5] c6 [eb6 c6] ab5]"),
    note("[[c6 c6] [c6 c6] [g5 g5] [c6 c6] [c6 c6] [g5 g5] [c6 g5] [c6 c6]]")
  ])
    .sound("square")
    .decay(.1)
    .sustain(.16)
    .release(.22)
    .room(.18)
    .roomsize(1.6)
    .gain(.54)


// Bass walks the same road: gait, eighths, sixteenths. Timbre
// climbs with it — triangle for bars 1-3, sawtooth from bar 4.
// .sound() is carried by EACH entry, not applied after the pick
// (which would overwrite it).
//
// Starts on TRIANGLE while APPROACH's bass is SQUARE — not an
// oversight: APPROACH uses square down low because triangle is
// busy with the counter-melody, the first thing the bridge drops.
// Bar 1 hands the low end back to triangle, -02's timbre ladder
// entered one rung later.
const bassBridgeUp =
  bridgeUpBar.pick([
    note("[c2  ~   c2  ~   g1  ~   c2  ~  ]").sound("triangle"),
    note("[f1  ~   f2  ~   c2  ~   f1  ~  ]").sound("triangle"),
    note("[ab1 ~   ab2 ab1 eb2 ~   ab2 ab1]").sound("triangle"),
    note("[db2 db3 db2 db3 ab1 db2 f2  ab2]").sound("sawtooth"),
    note("[ab1 ab2 ab1 ab2 eb2 ab1 c2  eb2]").sound("sawtooth"),
    note("[[c1 c1] c2 [c1 c1] c2 [c1 c1] c2 [g1 c2] c2]").sound("sawtooth")
  ])
    .decay(.2)
    .sustain(.12)
    .release(.07)
    .gain(.64)


// Reactor ignites. Bars 1-2 silent — the arp switches on rather
// than ramping, and placing that event two bars into a six-bar
// climb gives the climb a plot, not a slope.
//
// Chords are per-bar one-cycle patterns matching this bridge's
// own grid (Ab, Db, Ab, C5) — not reused from THE CHORD TABLE,
// since the bridge grid differs from the movement grid. Bar 6
// shares the open C/G/C voicing the melody and bass spell: a
// genuine unison across every pitched layer.
//
// Silent bars are NOT spelled with the bare `silence` export:
// mixing that raw core pattern into a pick() array alongside
// fully-built reactorVoiceLiteral() chains threw "Cannot read
// properties of undefined (reading 'value')" inside Strudel's own
// bundled Pattern.query (confirmed stack trace, never reaching
// this file's code) — silence's bare shape doesn't survive being
// array-mapped next to a fully chained pattern in a nested
// pick(). Fix: give silent bars the SAME shape via
// reactorVoiceLiteral() too, peak = 0 so .gain(saw.range(0, 0))
// is permanently zero — audibly silent, structurally identical to
// siblings. Their baked-in chord/root is otherwise inert, just
// matches this bridge's grid (Cm, Fm) for documentation.
const arpBridgeUp =
  bridgeUpBar.pick([
    reactorVoiceLiteral("[c4 eb4 g4]*2", 0, 0, 1),
    reactorVoiceLiteral("[f4 ab4 c5]*2", 0, 0, 1),
    reactorVoiceLiteral("[ab3 c4 eb4]*2", .14, 0.2, 1),
    reactorVoiceLiteral("[db4 f4 ab4]*4", .18, 0.5, 1),
    reactorVoiceLiteral("[c4 eb4 g4]*6", .22, 1.0, 1),
    reactorVoiceLiteral("[c4 g4 c5]*8", .26, 2.0, .5)
  ])


// Builds the drum kit for one bridge bar. .bank() stays on the
// 808 layers only — white noise isn't part of that bank.
//
// `att` attenuates all three layers at once: 1 through the
// accelerating bridge (acceleration alone raises pressure),
// decreasing bar by bar in the decelerating bridge to land on
// APPROACH's level with no step. Pinned to real drum gains:
//     skins .66 * 1   = .66  = ENGAGE's/OVERLOAD's skins
//     skins .66 * .73 = .482 ~ APPROACH's .48
//     hats  .46 * 1   = .46  = ENGAGE's/OVERLOAD's hats
//     hats  .46 * .73 = .336 ~ APPROACH's .34
// Both ends exact — why this file splits skins/hats into separate
// layers in all three movements (-02 kept them combined and took
// a small step on the hats at handoff). Change a movement's drum
// gain, redo this arithmetic.
//
// OVERLOAD needs no arithmetic of its own: skins at .66, exactly
// `.66*1`, the att both bridges use at their loud end — so the
// climb lands on OVERLOAD's level as cleanly as on ENGAGE's, and
// the decelerating bridge's first bar is also .66 coming back. A
// 2 -> 0 downbeat is thus a DENSITY step (hats 16->8, arp 24->12)
// with no volume step — the point of the manoeuvre.
const bridgeDrums = (skins, hats, noise, att = 1) =>
  stack(
    s(skins).bank("RolandTR808").gain(.66 * att),
    s(hats).bank("RolandTR808").gain(.46 * att),
    s(noise).hpf(9000).decay(.02).gain(.22 * att)
  )

// Acceleration is most audible here: hats 2 -> 16/bar, kick
// fills in, snare arrives and rolls.
const drumsBridgeUp =
  bridgeUpBar.pick([
    bridgeDrums("bd ~  ~  bd",             "hh ~ hh ~", "~"),
    bridgeDrums("bd ~  bd ~ ",             "hh ~ hh ~", "white"),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white*2"),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2"),
    bridgeDrums("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4"),
    bridgeDrums("bd bd bd bd, sd*4",       "hh*16",     "white*8")
  ])

const movementBridgeUp =
  stack(melodyBridgeUp, bassBridgeUp, arpBridgeUp, drumsBridgeUp)


// ============================================================
// DECELERATING BRIDGE  (ENGAGE / OVERLOAD -> APPROACH)
//
// Mirror of the previous bridge. ENGAGE straight into APPROACH
// read as a breakdown, not a release — a climb with no descent.
// So we wind SUBDIVISION back down.
//
// Onsets per bar:
//
//   bar       hats kick snare noise bass lead arp  wave
//   ENGAGE      8    3    2      4     8   12  12  saw
//   bridge 1    8    3    2      4    12   12  12  saw
//   bridge 2    6    2    2      2     8    8  12* saw
//   bridge 3    4    2    1      1     6    5  12* TRI
//   bridge 4    2    2    1      1     4    4  12* tri
//   APPROACH    2    2    1     0/1    4    4   0  sq+tri
//
// * bars 2-4's arp onsets are SILENT (peak = 0, same reasoning as
//   arpBridgeUp) — the shutdown is audible at the bar 1->2
//   boundary, not a fade to 0.
//
// Bar 1 keeps ENGAGE's hats but pushes BASS to 12: the fall
// starts by leaning forward, a gear change before a drop in
// energy. The arp then CUTS OUT ENTIRELY at bar 2 rather than
// winding down — a reactor shuts off, doesn't decelerate, and
// that silence is the clearest signal the fight is over. Bar 4
// matches APPROACH line for line — only the triangle
// counter-melody and sine are missing, swelling in next bar.
//
// ENTERING FROM OVERLOAD (2 -> 0): table starts from ENGAGE, and
// OVERLOAD gets that too — bar 1's hats (8 vs 16) and arp (12 vs
// 24) make the downbeat an audible density step, mirroring the
// climb's 0 -> 2. Nothing steps in VOLUME (bar 1 skins .66*1 =
// OVERLOAD's .66).
//
// Melody also descends in REGISTER bar after bar (octave 5-6 ->
// 4), reverse of the accelerating bridge.
//
// WHY 4 BARS NOT 6: a climb is waited for, a fall is felt. Six
// steps down would force repeated densities (8-6-6-4-4-2) and
// drag. Four bars = 7 s at 138 BPM, one hat step/bar (8-6-4-2),
// every step audible.
//
// GRID: Fm - Ab - Bb - Cm (iv - VI - VII - i). The accelerating
// bridge ends on an open fifth so it can land on any grid bar;
// here we want APPROACH ALREADY settled, so the cadence closes
// inside the bridge on bar 4's C. Pick preserves phase, so any
// chord after a C minor continues naturally.
//
// THE FIREWALL HOLDS HERE TOO. Bar 1 (Fm) carries the stride's
// Db; bar 3 (Bb: Bb D F) carries the only D natural in either
// bridge — two bars apart with bar 2 (Ab) as airlock, and the
// last bar is a plain C minor, so no Db/D pair can land adjacent
// whichever bar APPROACH resumes on.
//
// Bar 4 literally quotes APPROACH bar 1 (melody C-G-Ab-G, bass
// C/G, "bd ~ ~ bd" over "hh ~ hh ~"): the bridge becomes the next
// movement.
// ============================================================

const BRIDGE_DOWN_BARS = 4


// Bar index inside the decelerating bridge: 0..3.
const bridgeDownBar = signal((t) => {
  const i = Math.floor(Number(t) - _bridgeDownStart)
  return Math.min(BRIDGE_DOWN_BARS - 1, Math.max(0, i))
})


// ENGAGE's hammer un-syncopates: subdivisions drop away until
// only the bare stride remains, line falling an octave and a half.
const melodyBridgeDown =
  bridgeDownBar.pick([
    note("[[f5 f5] c6 [f5 ab5] c5 [db6 c6] ab5 [f5 ab5] c6]"),
    note("[ab5 [c6 ab5] eb5 ~   c5  [eb5 c5] ab4 ~ ]"),
    note("[bb4 bb4      ~   f4  d5  ~        bb4 ~ ]"),
    note("[c4  ~        g4  ~   ab4 ~        g4  ~ ]")
  ])
    .sound("square")
    .decay(.12)
    .sustain(.18)
    // Longer release than the accelerating bridge: tail builds
    // the fade into APPROACH, where the climb needed to stay crisp.
    .release(.3)
    .room(.24)
    .roomsize(1.8)
    .gain(.5)


// Bass walks bassBridgeUp's road backwards: sixteenths, eighths,
// then the gait — timbre comes down too, sawtooth -> triangle.
const bassBridgeDown =
  bridgeDownBar.pick([
    note("[[f1 f1] f2 [f1 f1] f2 [f1 f1] f2 [c2 f2] f1]").sound("sawtooth"),
    note("[ab1 ab2 ab1 ab2 eb2 ab1 c2  eb2]").sound("sawtooth"),
    note("[bb1 ~   bb2 bb1 f2  ~   bb2 d2 ]").sound("triangle"),
    note("[c2  ~   c2  ~   g1  ~   c2  ~  ]").sound("triangle")
  ])
    .decay(.24)
    .sustain(.14)
    .release(.1)
    .gain(.62)


// One bar of reactor, then nothing. Same note count as ENGAGE
// (12/bar, no step from movement 2) but half OVERLOAD's (a step
// from movement 3 — see entering from OVERLOAD above). fm drops
// to its lowest setting: the reactor goes dull before it goes
// quiet. Bars 2-4 silent via the same reactorVoiceLiteral-shaped,
// peak = 0 treatment as arpBridgeUp (bare `silence` next to built
// reactorVoiceLiteral() chains in this pick() is the confirmed
// Strudel bug site). Chords/roots follow this bridge's grid
// (Ab, Bb, Cm) for documentation only.
const arpBridgeDown =
  bridgeDownBar.pick([
    reactorVoiceLiteral("[f4 ab4 c5]*4", .18, 0.1, 1),
    reactorVoiceLiteral("[ab3 c4 eb4]*4", 0, 0, 1),
    reactorVoiceLiteral("[bb3 d4 f4]*4", 0, 0, 1),
    reactorVoiceLiteral("[c4 eb4 g4]*4", 0, 0, 1)
  ])


// Hats come down 8 -> 6 -> 4 -> 2, kick empties out, snare fades,
// attenuation follows the density drop to land exactly on
// APPROACH's level (see bridgeDrums for the arithmetic).
//
// Expected artefact, not a bug: on a downbeat where the step
// changes, the PREVIOUS step's note still triggers once on top of
// the new one (pick samples just before the bar line) — inaudible
// in the accelerating bridge (both hits share gain), audibly
// louder here since gains differ, but lands on the downbeat where
// the kick masks it. Nothing to fix; it's why the WRITING RULE
// matters — a bar starting on a REST would let it bleed through.
const drumsBridgeDown =
  bridgeDownBar.pick([
    bridgeDrums("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4", 1),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2", .92),
    bridgeDrums("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white",   .82),
    bridgeDrums("bd ~  ~  bd, ~ ~  sd ~ ", "hh ~ hh ~", "white",   .73)
  ])

const movementBridgeDown =
  stack(
    melodyBridgeDown,
    bassBridgeDown,
    arpBridgeDown,
    drumsBridgeDown
  )


// ============================================================
// STATE MACHINE
//
// THE STATE ACTUALLY PLAYING: 0/1/2 like the slider, plus 3
// (accelerating bridge) and 4 (decelerating bridge).
//
// -02's machine, unchanged in logic, renamed only where movement
// names differ. Deliberately NOT reinvented: every branch below
// exists because a specific bug surfaced while -02 was written.
//
// signal(fn) receives time in cycles, letting us count bars since
// the switch. As a pick selector it samples at bar start, so
// bridge steps land exactly on downbeats.
//
// Math.ceil pins the bridge start to the NEXT BAR so the current
// bar finishes cleanly first — without it, steps land mid-bar.
//
// ARMING: a bridge arms only if the OUTGOING STATE suits it —
// APPROACH (0) for accelerating, ENGAGE/OVERLOAD (1/2) for
// decelerating — never the SLIDER value: between the gesture and
// Math.ceil there's a bar-fragment where the old movement still
// sounds but the slider already reads new. A fast 0->1->0 round
// trip in that window would otherwise arm a decelerating bridge
// out of a texture that never accelerated. _prevState fixes this
// — any round trip inside a bridge or its waiting window disarms
// both and switches cleanly.
//
// DESTINATION-AGNOSTIC CLIMB: "APPROACH spooling up", not
// "APPROACH becoming ENGAGE" — arms for ANY non-zero slider from
// a played APPROACH, exits via `e = mv` to whatever the slider
// reads after six bars. 0->2 gets the same ramp as 0->1; swapping
// the destination mid-climb RETARGETS the landing rather than
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
//    still 0, so mv === 0 fails both arm guards and APPROACH
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
      _bridgeUpStart = Math.ceil(x)                   // APPROACH -> ENGAGE / OVERLOAD
      _bridgeDownStart = -Infinity
      _bridgeFrom = 0
    } else if (mv === 0 && (_prevState === 1 || _prevState === 2)) {
      _bridgeDownStart = Math.ceil(x)                 // ENGAGE / OVERLOAD -> APPROACH
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
    else e = 0                                               // APPROACH settled
  } else {
    if (x < _bridgeUpStart) e = 0                            // finish current bar
    else if (x - _bridgeUpStart < BRIDGE_UP_BARS) e = 3      // accelerating bridge
    else e = mv                                              // ENGAGE / OVERLOAD settled
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
// so a slowcat phrase plays bar 1 forever (c4 c4 c4...). The
// only workaround is cramming all 8 bars into one 14 s cycle,
// making a switch take up to 14 s.
//
// pick restarts nothing — routes to the chosen pattern, which
// keeps running on ITS OWN clock:
//  - Phase preserved: switching on bar 5 enters the new movement
//    at ITS bar 5. Shared grid means harmony continues with no
//    seam — the real source of the smoothness. Also lets the
//    reactor arp keep its place in the chord table across a
//    switch: jumping into OVERLOAD on bar 6 continues on Ab like
//    everything else, no restart on Cm.
//  - Structure preserved: pick takes structure from the chosen
//    pattern, not the selector, so each bar's 8 eighths stay
//    intact.
//
// SWITCH LATENCY: slider() reads live at query time, not frozen
// at cycle start. Instant switches take effect on the next
// scheduled note — one eighth, ~0.22 s at 138 BPM.
//
// THE DELIBERATE EXCEPTIONS: every switch leaving/entering the
// approach goes through a bridge.
//   0 -> 1, 0 -> 2 : accelerating, 6 bars (10.4 s). Heard right
//            away (next bar, 1.7 s), full accelerate takes ~12 s.
//            0 -> 2 is the same climb, landing on OVERLOAD.
//   1 -> 0, 2 -> 0 : decelerating, 4 bars (7 s), ~8.7 s total —
//            shorter on purpose (see that section). From OVERLOAD
//            the wind-down opens on ENGAGE's density (hats
//            16 -> 8, arp 24 -> 12 on the downbeat): 2 -> 0
//            chains through 1.
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
// stay dry — a low tail would muddy the switch. The arp is the
// opposite exception: release 0.05 s, so when it stops it stops,
// exactly what the decelerating bridge wants.
//
// Two syntax traps:
//  - Argument order: it's selector.pick([...]). The reverse
//    (stack(...).pick(selector)) loops infinitely at evaluation.
//  - slider() returns a Pattern, not a number — raw arithmetic on
//    it silently yields NaN then an AudioParam error. Always go
//    through .range(min, max). Doesn't apply to saw.range(...) in
//    the reactor voice: saw is a signal, plain-number .range() is
//    the intended use there.
//
// Finally .mul(gain(volume)), not .gain(volume): .gain()
// overwrites each layer's own gain (mix disappears), .mul()
// multiplies it — including the reactor arp's saw envelope, the
// only reason that envelope survives the master fader at all.
// ============================================================

// Array order IS the state encoding: 0 APPROACH, 1 ENGAGE,
// 2 OVERLOAD, 3 accelerating bridge, 4 decelerating bridge. pick
// is positional and silent if off by one — an entry inserted
// mid-array would play the wrong movement with no error.
playedState
  .pick([
    movementApproach,
    movementEngage,
    movementOverload,
    movementBridgeUp,
    movementBridgeDown
  ])
  .lpf(filterOpenness.range(500, 12000))
  .mul(gain(volume))
