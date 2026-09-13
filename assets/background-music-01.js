// ============================================================
// TYLED
// Action / stratégie 2D vue du dessus
// ------------------------------------------------------------
// Dans l'esprit des OST 8-bit de la NES : do majeur franc,
// lead carré sautillant, basse triangle qui marche, percussion
// bruitée. Thème original, décliné en trois épisodes.
// ------------------------------------------------------------
// MOUVEMENT 1 : joueurs éloignés   -> stratégie / observation
// MOUVEMENT 2 : joueurs proches    -> tension / combat
// MOUVEMENT 3 : fin de partie      -> conclusion / danger
//
// Le contrôle "MOUVEMENT" en haut permet de changer de partie.
// Les deux bascules entre STRATÉGIE et TENSION ne se font pas
// d'un coup, elles passent par un PONT (voir tout en bas) :
//   0 -> 1 : PONT D'ACCÉLÉRATION,  6 mesures, la machine monte
//            en régime (subdivisions 2 -> 8).
//   1 -> 0 : PONT DE DÉCÉLÉRATION, 4 mesures, la machine
//            redescend (subdivisions 8 -> 2).
// Toutes les autres bascules restent immédiates.
// ============================================================


// ============================================================
// TABLEAU DE CONTRÔLE
// ============================================================

// 0 = STRATÉGIE
// 1 = TENSION
// 2 = FIN DE PARTIE

const mouvement = slider(0, 0, 2, 1)


// Volume général
const volume = slider(0.8, 0, 1, 0.01)


// Filtre stratégique
// 0 = très fermé (étouffé, lointain)
// 1 = très ouvert (brillant, proche)
// Il agit sur TOUT le mix, percussion comprise : c'est voulu,
// le morceau doit pouvoir s'éteindre d'un seul geste.
const ouverture = slider(0.45, 0, 1, 0.01)


// ============================================================
// RÉGLAGES GÉNÉRAUX
//
// ATTENTION : setcpm compte des CYCLES par minute, pas des
// noires. Ici UN CYCLE = UNE MESURE de quatre temps à 150 BPM,
// soit 150/4 = 37.5 cycles par minute, soit 1,6 s par mesure.
//
// La phrase complète fait toujours 8 mesures, mais elle s'écrit
// comme un slowcat < ... > : 8 mesures = 8 cycles. C'est
// l'écriture naturelle de Strudel, et c'est elle qui rend le
// changement de mouvement rapide (voir tout en bas).
//
// Conséquence pratique : chaque ligne mélodique s'écrit comme
// < [mesure1] [mesure2] ... [mesure8] >, 8 croches par mesure.
// Les percussions s'écrivent sur UNE mesure et se répètent
// toutes seules, un cycle = une mesure : PAS de .fast(8).
// ============================================================

setcpm(150 / 4)


// ============================================================
// LE MOTIF
//
// Toute la musique tient sur un même geste de trois notes :
//     mi - mi - sol      (note répétée, puis saut)
// C'est le "hop" du robot. On le retrouve tel quel au
// mouvement 1, séquencé et syncopé au mouvement 2, renversé
// (note répétée puis chute) au mouvement 3.
//
// Les trois mouvements partagent le MÊME tempo, la MÊME
// tonalité et la MÊME grille de 8 mesures
// (C - G - F - C - Am - F - G - C). Ce n'est pas un détail
// d'écriture : c'est ce qui rend les bascules fluides, puisque
// la mesure 5 de TENSION prolonge harmoniquement la mesure 4
// de STRATÉGIE.
// ============================================================


// ============================================================
// MOUVEMENT 1 : STRATÉGIE
// Les robots sont loin. Question / réponse, beaucoup d'air.
// Grille : C - G - F - C - Am - F - G - C.
// Les 4 premières mesures posent le motif, les 4 suivantes le
// reprennent une tierce plus haut avant de retomber.
// ============================================================

const melodyStrategie =
  note(`<
    [e5 e5 ~  g5 e5 ~  c5 ~ ]
    [d5 ~  e5 d5 ~  g4 ~  ~ ]
    [f5 f5 ~  a5 f5 ~  d5 ~ ]
    [e5 ~  d5 c5 ~  ~  ~  ~ ]
    [e5 e5 ~  g5 a5 ~  g5 ~ ]
    [f5 ~  e5 d5 ~  e5 ~  ~ ]
    [g5 g5 ~  b5 a5 ~  f5 ~ ]
    [e5 ~  d5 ~  c5 ~  ~  ~ ]
  >`)
    .sound("square")
    .decay(.14)
    .sustain(.25)
    // Release long + un peu de réverbération et d'écho : c'est
    // la queue sonore qui fait le fondu quand on change de
    // mouvement (voir la note sur les transitions, en bas).
    .release(.35)
    .room(.25)
    .roomsize(2)
    .delay(.22)
    .delaytime(.15)
    .delayfeedback(.3)
    .gain(.55)

const bassStrategie =
  note(`<
    [c2 ~ c3 ~ g2 ~ c3 ~]
    [g1 ~ g2 ~ d2 ~ g2 ~]
    [f1 ~ f2 ~ c2 ~ f2 ~]
    [c2 ~ c3 ~ g2 ~ e2 ~]
    [a1 ~ a2 ~ e2 ~ a2 ~]
    [f1 ~ f2 ~ c2 ~ f2 ~]
    [g1 ~ g2 ~ d2 ~ b2 ~]
    [c2 ~ c3 ~ g2 ~ g2 ~]
  >`)
    .sound("triangle")
    .decay(.3)
    .sustain(.2)
    // La basse garde un release court : une queue de basse
    // rendrait la bascule boueuse au lieu de fluide.
    .release(.12)
    .gain(.7)

// Nappe d'accords très en retrait : elle pose la grille sans
// jamais tirer l'oreille. Un accord par mesure.
const padStrategie =
  note(`<
    [c4,e4,g4] [b3,d4,g4] [a3,c4,f4] [c4,e4,g4]
    [a3,c4,e4] [a3,c4,f4] [b3,d4,g4] [c4,e4,g4]
  >`)
    .sound("triangle")
    // Attaque douce : la nappe entre en gonflant, jamais en
    // claquant, y compris quand elle arrive par une bascule.
    .attack(.12)
    .decay(.4)
    .sustain(.35)
    .release(.5)
    .room(.3)
    .gain(.18)

const drumsStrategie =
  stack(
    s("bd ~ ~ ~, ~ ~ sd ~, hh ~ hh ~")
      .bank("RolandTR808")
      .gain(.45),

    // Un souffle une mesure sur deux, juste pour respirer.
    s("<~ [~ ~ ~ ~ ~ ~ ~ white]>")
      .hpf(9000)
      .decay(.02)
      .gain(.22)
  )

const mouvementStrategie =
  stack(melodyStrategie, bassStrategie, padStrategie, drumsStrategie)


// ============================================================
// MOUVEMENT 2 : TENSION
// Les robots se rapprochent. Le motif est séquencé en
// doubles-croches et monte : mi-mi-sol devient une relance qui
// grimpe à chaque mesure. Basse en octaves, caisse claire tous
// les deux temps, roulement toutes les 4 mesures.
// ============================================================

const melodyAction =
  note(`<
    [[e5 e5] g5 [a5 g5] e5 [g5 a5] b5 ~ [b5 a5]]
    [g5 [a5 b5] c6 ~ [b5 g5] e5 [d5 e5] g5]
    [[f5 f5] a5 [c6 a5] f5 [a5 c6] d6 ~ [c6 a5]]
    [a5 [g5 f5] e5 ~ [d5 e5] d5 c5 ~]
    [[e5 e5] g5 [c6 b5] a5 [g5 a5] b5 ~ [c6 b5]]
    [a5 [b5 c6] d6 ~ [c6 a5] g5 [e5 g5] a5]
    [[b5 b5] d6 [c6 b5] g5 [a5 b5] c6 ~ [d6 c6]]
    [b5 [a5 g5] e5 ~ [d5 e5] g5 c5 ~]
  >`)
    .sound("square")
    .decay(.08)
    .sustain(.12)
    // Plus court qu'au mouvement 1 (le débit est double), mais
    // assez long pour déborder sur la bascule.
    .release(.22)
    .room(.18)
    .roomsize(1.5)
    .gain(.62)

const bassAction =
  note(`<
    [c2 c3 c2 c3 g2 c3 g2 b2]
    [g1 g2 g1 g2 d2 g2 b2 d3]
    [f1 f2 f1 f2 c2 f2 a2 c3]
    [g1 g2 b1 d2 c2 c3 g2 e2]
    [a1 a2 a1 a2 e2 a2 c3 e3]
    [f1 f2 f1 f2 c2 f2 a2 c3]
    [g1 g2 g1 g2 d2 g2 b2 d3]
    [g1 g2 b1 d2 c2 c3 c2 g2]
  >`)
    .sound("triangle")
    .decay(.16)
    .sustain(.1)
    .release(.06)
    .gain(.8)

const drumsAction =
  stack(
    s("bd ~ bd ~, ~ sd ~ sd, hh*8")
      .bank("RolandTR808")
      .gain(.7),

    // Relance de caisse claire sur la 4e mesure de chaque carrure.
    // Un cycle = une mesure, donc ce slowcat de 4 éléments tombe
    // bien toutes les 4 mesures (mesures 4 et 8 de la phrase).
    s("<~!3 [~ ~ sd sd]>")
      .bank("RolandTR808")
      .gain(.6),

    s("white*4")
      .hpf(9000)
      .decay(.02)
      .gain(.28)
  )

const mouvementAction =
  stack(melodyAction, bassAction, drumsAction)


// ============================================================
// MOUVEMENT 3 : FIN DE PARTIE
// Le motif est renversé : note répétée puis CHUTE. Registre
// plus haut, une deuxième voix carrée martèle une pédale
// syncopée, la basse passe en dents de scie, et un fa dièse
// vient tordre les mesures 4 et 7 (emprunt à ré majeur) juste
// avant la retombée.
// ============================================================

const melodyFinal =
  note(`<
    [[c6 c6] a5 [g5 a5] c6 [b5 a5] g5 [e5 g5] a5]
    [[g5 g5] e5 [d5 e5] g5 [f5 e5] d5 [c5 d5] e5]
    [[a5 a5] f5 [e5 f5] a5 [g5 f5] e5 [d5 e5] f5]
    [[g5 a5] b5 [c6 d6] e6 [d6 c6] b5 [a5 f#5] g5]
    [[c6 c6] a5 [g5 a5] c6 [d6 c6] b5 [g5 b5] d6]
    [[e6 e6] c6 [b5 c6] e6 [d6 c6] b5 [a5 b5] c6]
    [[d6 d6] b5 [a5 b5] d6 [c6 b5] a5 [f#5 a5] b5]
    [[c6 b5] a5 [g5 f5] e5 [d5 e5] g5 c6 ~]
  >`)
    .sound("square")
    .decay(.07)
    .sustain(.1)
    .release(.22)
    .room(.18)
    .roomsize(1.5)
    .gain(.65)

// Deuxième voix : pédale syncopée, elle serre le rythme.
const contreFinal =
  note(`<
    [e5 ~ ~ e5 ~ ~ e5 ~]
    [c5 ~ ~ c5 ~ ~ c5 ~]
    [f5 ~ ~ f5 ~ ~ f5 ~]
    [d5 ~ ~ d5 ~ ~ b4 ~]
    [e5 ~ ~ e5 ~ ~ g5 ~]
    [g5 ~ ~ g5 ~ ~ e5 ~]
    [f#5 ~ ~ f#5 ~ ~ d5 ~]
    [e5 ~ ~ d5 ~ ~ c5 ~]
  >`)
    .sound("square")
    .decay(.06)
    .sustain(0)
    .release(.14)
    .room(.15)
    .gain(.3)

const bassFinal =
  note(`<
    [a1 a1 e2 a1 c2 c2 g2 c2]
    [c2 c2 g2 c2 g1 g1 d2 g1]
    [f1 f1 c2 f1 d2 d2 a2 d2]
    [g1 g1 d2 g1 g1 g2 d2 b1]
    [a1 a1 e2 a1 g1 g1 d2 g1]
    [c2 c2 g2 c2 e2 e2 b1 e2]
    [d2 d2 a2 d2 g1 g1 d2 g1]
    [c2 c2 g2 c2 g1 g2 c2 c3]
  >`)
    .sound("sawtooth")
    .decay(.14)
    .sustain(.08)
    .release(.04)
    .gain(.75)

const drumsFinal =
  stack(
    s("bd bd ~ bd, ~ sd ~ sd, hh*8")
      .bank("RolandTR808")
      .gain(.8),

    // Roulement complet toutes les 4 mesures : ça pousse vers la fin.
    s("<~!3 [sd sd sd sd]>")
      .bank("RolandTR808")
      .gain(.5),

    s("white*8")
      .hpf(9000)
      .decay(.015)
      .gain(.3)
  )

const mouvementFinal =
  stack(melodyFinal, contreFinal, bassFinal, drumsFinal)


// ============================================================
// PONT D'ACCÉLÉRATION  (STRATÉGIE -> TENSION)
//
// Passer de l'observation au combat d'un seul coup était trop
// brutal : on veut ENTENDRE la machine monter en régime. Le
// pont dure 6 mesures (9,6 s) et n'accélère qu'une chose, mais
// il l'accélère partout à la fois : la SUBDIVISION.
//
//   mesure 1 : croches espacées,  charleston 2 par mesure
//   mesure 2 : première double,   charleston 4
//   mesure 3 : caisse claire,     charleston 4
//   mesure 4 : doubles installées, charleston 6
//   mesure 5 : presque que des doubles, charleston 8
//   mesure 6 : doubles pleines + roulement -> on tombe dans TENSION
//
// La grille monte C - Am - F - G - G - G7 : le sol 7 de la
// dernière mesure appelle le do de TENSION, donc l'arrivée
// sonne comme une résolution et pas comme un raccord.
//
// Chaque mesure du pont est un pattern d'UN SEUL cycle (pas de
// slowcat) : on choisit laquelle jouer avec `mesurePont`. C'est
// volontaire — un pattern d'un cycle se joue à l'identique quel
// que soit le cycle absolu, donc le pont se déroule toujours
// dans l'ordre 1, 2, 3, 4, 5, 6, où qu'on soit dans le morceau.
// ============================================================

const PONT_MESURES = 6


// Mémoire des bascules. Ces quatre variables sont le SEUL état
// mutable du morceau : Strudel n'a aucune notion de "valeur
// précédente du slider", il faut donc la tenir soi-même.
//
// -Infinity = "pont non armé". C'est la valeur neutre, pas
// Infinity : avec -Infinity les tests d'avancement du pont
// (x < debut, x - debut < LONGUEUR) sont tous les deux faux, donc
// l'état demandé sort IMMÉDIATEMENT. Avec Infinity, x < debut
// était toujours vrai et une arrivée en TENSION qui ne venait pas
// de STRATÉGIE (2 -> 1) restait bloquée sur STRATÉGIE
// indéfiniment. Bug corrigé, testé au REPL.
//
// Au chargement, _etatPrecedent vaut 0 : si le slider est déjà
// sur 1 à l'évaluation, le morceau part donc par le pont montant
// et non directement en TENSION. C'est voulu — en jeu on démarre
// toujours en STRATÉGIE, et au REPL c'est la montée qu'on veut
// entendre.
let _mvPrecedent = null
let _etatPrecedent = 0
let _pontDebut = -Infinity
let _pontDescDebut = -Infinity


// Lit la valeur courante du slider. mouvement est un Pattern :
// on l'interroge sur un intervalle minuscule et on prend la
// valeur. Elle est lue en direct, pas figée en début de cycle.
const lireMouvement = (t) => {
  const e = mouvement.queryArc(t, t + 1e-6)
  return e.length ? e[0].value : 0
}


// Numéro de mesure à l'intérieur du pont montant : 0..5.
const mesurePont = signal((t) => {
  const i = Math.floor(Number(t) - _pontDebut)
  return Math.min(PONT_MESURES - 1, Math.max(0, i))
})


// Le "hop" de STRATÉGIE, mesure après mesure, rongé par les
// doubles-croches jusqu'à devenir la relance de TENSION.
const melodyPont =
  mesurePont.pick([
    note("[e5 e5 ~  g5 ~  ~  e5 ~ ]"),
    note("[e5 e5 ~  g5 [a5 g5] ~  e5 ~ ]"),
    note("[f5 f5 ~  a5 [g5 f5] e5 [d5 e5] ~ ]"),
    note("[g5 [g5 a5] b5 [a5 g5] e5 [g5 a5] b5 ~ ]"),
    note("[[a5 b5] c6 [b5 a5] g5 [a5 b5] c6 [d6 c6] b5]"),
    note("[[b5 c6] d6 [c6 b5] a5 [b5 c6] d6 [e6 d6] c6]")
  ])
    .sound("square")
    .decay(.1)
    .sustain(.18)
    .release(.28)
    .room(.2)
    .roomsize(1.6)
    .gain(.6)


// La basse suit le même chemin : blanches, puis noires, puis
// octaves en croches, puis pédale de sol qui pousse.
const bassPont =
  mesurePont.pick([
    note("[c2 ~  ~  ~  g2 ~  ~  ~ ]"),
    note("[a1 ~  a2 ~  e2 ~  a2 ~ ]"),
    note("[f1 ~  f2 ~  c2 ~  f2 ~ ]"),
    note("[g1 g2 g1 g2 d2 g2 d2 g2]"),
    note("[g1 g2 b1 d2 g1 g2 b1 d2]"),
    note("[g1 g2 g1 g2 b1 d2 f2 g2]")
  ])
    .sound("triangle")
    .decay(.2)
    .sustain(.14)
    .release(.08)
    .gain(.76)


// Fabrique la batterie d'une mesure du pont. On garde .bank()
// sur la seule couche 808 : le bruit blanc n'appartient pas à
// la banque et serait cassé par un .bank() global.
//
// `att` atténue les trois couches d'un coup. Il vaut 1 pour tout
// le pont montant (l'accélération suffit à faire monter la
// pression) et décroît mesure après mesure dans le pont
// descendant, pour arriver au niveau de STRATÉGIE sans marche.
const batteriePont = (peau, charley, souffle, att = 1) =>
  stack(
    s(peau).bank("RolandTR808").gain(.72 * att),
    s(charley).bank("RolandTR808").gain(.5 * att),
    s(souffle).hpf(9000).decay(.02).gain(.26 * att)
  )

// C'est ici que l'accélération s'entend le plus franchement :
// le charleston passe de 2 à 8 coups par mesure, la grosse
// caisse se remplit, la caisse claire arrive puis roule.
const drumsPont =
  mesurePont.pick([
    batteriePont("bd ~  ~  ~ ",             "hh ~ hh ~", "~"),
    batteriePont("bd ~  ~  ~ ",             "hh*4",      "white"),
    batteriePont("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white*2"),
    batteriePont("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2"),
    batteriePont("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4"),
    batteriePont("bd bd bd bd, sd*4",       "hh*8",      "white*8")
  ])

const mouvementPont =
  stack(melodyPont, bassPont, drumsPont)


// ============================================================
// PONT DE DÉCÉLÉRATION  (TENSION -> STRATÉGIE)
//
// Le miroir du précédent. Retomber d'un seul coup de TENSION
// (charleston plein, grosse caisse qui pousse, mélodie en
// doubles) dans l'air de STRATÉGIE ne s'entendait pas comme un
// apaisement mais comme une panne : la montée existait, la
// descente non, et l'asymétrie s'entendait. On redescend donc
// la même chose qu'on avait montée : la SUBDIVISION.
//
//   mesure 1 : charleston 8, doubles encore là, le roulement part
//   mesure 2 : charleston 6, la caisse claire s'allège
//   mesure 3 : charleston 4, mélodie en croches franches
//   mesure 4 : charleston 2, batterie et basse de STRATÉGIE
//
// POURQUOI 4 MESURES ET NON 6 :
// une montée se fait attendre, une retombée se fait sentir. Six
// paliers à la descente obligeraient à répéter des densités
// (8-6-6-4-4-2) et le morceau traînerait juste au moment où il
// doit se détendre. Quatre mesures = 6,4 s, un palier de
// charleston par mesure (8-6-4-2), chaque marche s'entend, et
// la longueur tombe pile sur la moitié de la carrure de 8.
//
// GRILLE : Am - F - G - C, soit vi - IV - V - I.
// Le pont montant finit sur un G7 parce que la RÉSOLUTION doit
// se faire à l'entrée de TENSION. Ici c'est l'inverse : on veut
// être DÉJÀ posé quand STRATÉGIE reprend, donc la cadence se
// referme à l'intérieur du pont, sur le do de la mesure 4. Comme
// pick conserve la phase, STRATÉGIE peut reprendre sur n'importe
// quelle mesure de sa grille — or après un do, tous les accords
// de la grille (C, G, F, Am) sont un enchaînement naturel, alors
// qu'après un G7 seul le do l'aurait été.
//
// Et la mesure 4 cite littéralement la mesure 1 de STRATÉGIE
// (mélodie mi-mi-sol, basse do/sol, "bd ~ ~ ~ / hh ~ hh ~") :
// le pont ne s'arrête pas, il devient le mouvement suivant.
// ============================================================

const PONT_DESC_MESURES = 4


// Numéro de mesure à l'intérieur du pont descendant : 0..3.
const mesurePontDesc = signal((t) => {
  const i = Math.floor(Number(t) - _pontDescDebut)
  return Math.min(PONT_DESC_MESURES - 1, Math.max(0, i))
})


// La relance de TENSION se dé-syncope : les doubles-croches
// tombent une par une jusqu'à ne plus laisser que le "hop".
const melodyPontDesc =
  mesurePontDesc.pick([
    note("[[a5 b5] c6 [b5 a5] g5 [a5 g5] e5 a5 ~ ]"),
    note("[f5 [a5 g5] f5 ~  a5 [c6 a5] f5 ~ ]"),
    note("[b5 ~  a5 g5 d5 g5 ~  d5]"),
    note("[e5 e5 ~  g5 e5 ~  c5 ~ ]")
  ])
    .sound("square")
    .decay(.12)
    .sustain(.2)
    // Release un peu plus long que dans le pont montant : c'est
    // la queue sonore qui fabrique le fondu vers l'air de
    // STRATÉGIE, alors qu'à la montée elle devait rester nette.
    .release(.32)
    .room(.22)
    .roomsize(1.8)
    .gain(.56)


// La basse fait le chemin inverse de bassPont : octaves en
// croches, puis noires, puis la blanche do/sol de STRATÉGIE.
const bassPontDesc =
  mesurePontDesc.pick([
    note("[a1 a2 a1 a2 e2 a2 c3 e3]"),
    note("[f1 f2 f1 f2 c2 ~  f2 ~ ]"),
    note("[g1 ~  g2 ~  d2 ~  g2 ~ ]"),
    note("[c2 ~  ~  ~  g2 ~  ~  ~ ]")
  ])
    .sound("triangle")
    .decay(.24)
    .sustain(.16)
    .release(.1)
    .gain(.72)


// Le charleston redescend 8 -> 6 -> 4 -> 2, la grosse caisse se
// vide, la caisse claire s'efface, et l'atténuation accompagne
// la chute de densité pour arriver au niveau de STRATÉGIE.
//
// Relevé au REPL : sur un temps fort de changement de palier, la
// note du palier PRÉCÉDENT est encore déclenchée une fois, en
// plus de la nouvelle (pick échantillonne le signal juste avant
// la barre de mesure). C'est vrai depuis toujours pour le pont
// montant, où ça ne s'entend pas puisque les deux coups tombent
// au même instant avec le même gain. Ici les gains diffèrent
// d'un palier à l'autre : le doublon tombe sur le temps fort, où
// la grosse caisse le masque. Rien à corriger — mais à savoir
// avant d'écrire une mesure de pont qui commencerait par un
// silence : le palier précédent y déborderait à découvert.
const drumsPontDesc =
  mesurePontDesc.pick([
    batteriePont("bd ~  bd bd, ~ sd ~  sd", "hh*8",      "white*4", 1),
    batteriePont("bd ~  bd ~ , ~ sd ~  sd", "hh*6",      "white*2", .9),
    batteriePont("bd ~  bd ~ , ~ sd ~  ~ ", "hh*4",      "white",   .78),
    batteriePont("bd ~  ~  ~ ",             "hh ~ hh ~", "~",       .66)
  ])

const mouvementPontDesc =
  stack(melodyPontDesc, bassPontDesc, drumsPontDesc)


// ============================================================
// MACHINE D'ÉTAT
//
// L'ÉTAT RÉELLEMENT JOUÉ : 0, 1, 2 comme le slider, plus 3
// ("pont montant") et 4 ("pont descendant").
//
// signal(fn) donne un pattern continu dont fn reçoit le temps
// en cycles — c'est ce qui permet de compter les mesures
// écoulées depuis la bascule. Vérifié au REPL : utilisé comme
// sélecteur de pick, le signal est échantillonné au début de
// chaque mesure, donc les paliers du pont tombent pile sur les
// temps forts.
//
// Math.ceil cale le départ du pont sur la MESURE SUIVANTE : on
// termine proprement la mesure en cours dans le mouvement de
// départ, puis le pont démarre sur un temps fort. Sans ça le
// pont commencerait au milieu d'une mesure et les paliers
// seraient décalés.
//
// ARMEMENT : un pont ne s'arme que si l'ÉTAT SORTANT est
// exactement son état de départ (0 pour le montant, 1 pour le
// descendant) — et surtout pas si le SLIDER vaut 0 ou 1, ce qui
// n'est pas la même chose. Entre le geste sur le slider et
// Math.ceil il reste un bout de mesure pendant lequel l'ancien
// mouvement joue encore ; tester le slider y verrait déjà le
// nouveau. Un aller-retour rapide 0 -> 1 -> 0 dans cette fenêtre
// armerait alors un pont descendant depuis une texture qui n'a
// jamais accéléré (et symétriquement). Avec _etatPrecedent, tout
// aller-retour à l'intérieur d'un pont ou de sa fenêtre d'attente
// désarme les deux ponts et bascule net.
const etat = signal((t) => {
  const x = Number(t)
  const mv = lireMouvement(x)

  if (mv !== _mvPrecedent) {
    if (mv === 1 && _etatPrecedent === 0) {
      _pontDebut = Math.ceil(x)                       // STRATÉGIE -> TENSION
      _pontDescDebut = -Infinity
    } else if (mv === 0 && _etatPrecedent === 1) {
      _pontDescDebut = Math.ceil(x)                   // TENSION -> STRATÉGIE
      _pontDebut = -Infinity
    } else {
      _pontDebut = -Infinity                          // tout le reste : immédiat
      _pontDescDebut = -Infinity
    }
    _mvPrecedent = mv
  }

  let e
  if (mv === 1) {
    if (x < _pontDebut) e = 0                         // fin de la mesure en cours
    else if (x - _pontDebut < PONT_MESURES) e = 3     // pont montant
    else e = 1                                        // TENSION installée
  } else if (mv === 0) {
    if (x < _pontDescDebut) e = 1                     // fin de la mesure en cours
    else if (x - _pontDescDebut < PONT_DESC_MESURES) e = 4   // pont descendant
    else e = 0                                        // STRATÉGIE installée
  } else {
    e = mv
  }

  _etatPrecedent = e
  return e
})


// ============================================================
// SÉLECTION DU MOUVEMENT  —  pick, PAS pickRestart
//
// C'est ici que se joue la fluidité des transitions.
//
// POURQUOI PLUS pickRestart :
// pickRestart relance le pattern choisi à CHAQUE cycle. Vérifié
// au REPL : sur une phrase écrite en slowcat, il ne joue
// éternellement que la mesure 1 (on obtient c4 c4 c4... au lieu
// de c4 d4 e4 f4...). Une vieille version contournait ça en
// tassant les 8 mesures dans un seul cycle de 12,8 s — ce qui
// rendait le changement de mouvement lent jusqu'à 12,8 s.
//
// pick, lui, ne relance rien : il se contente d'aiguiller vers
// le pattern choisi, qui continue sur SA propre horloge. Deux
// conséquences, toutes deux vérifiées au REPL :
//
//  - Phase conservée. En basculant à la mesure 5, le nouveau
//    mouvement entre à SA mesure 5, pas au début de sa phrase.
//    Comme les trois mouvements partagent la même grille
//    d'accords, l'harmonie se poursuit sans couture : c'est ça,
//    la vraie source de la fluidité.
//
//  - Structure conservée. pick prend sa structure du pattern
//    choisi, pas du sélecteur : les 8 croches de chaque mesure
//    sortent intactes.
//
// LATENCE DE BASCULE : slider() est lu en direct, au moment où
// l'ordonnanceur interroge le pattern — et non figé en début de
// cycle (vérifié : une requête en milieu de cycle voit tout de
// suite la nouvelle valeur). Les bascules immédiates prennent
// donc effet à la PROCHAINE note programmée, soit une croche,
// environ 0,2 s.
//
// DEUX EXCEPTIONS, VOULUES : les allers-retours entre STRATÉGIE
// et TENSION passent par un pont.
//   0 -> 1 : pont montant, 6 mesures. On entend le changement
//            tout de suite (au plus tard à la mesure suivante,
//            1,6 s), mais la bascule met ~11 s à se terminer, le
//            temps que le rythme accélère pour de bon.
//   1 -> 0 : pont descendant, 4 mesures, ~7,5 s au total. Plus
//            court exprès : voir la section PONT DE
//            DÉCÉLÉRATION.
// Les autres bascules — 0 -> 2, 1 -> 2, 2 -> 1, 2 -> 0 —
// restent immédiates, et bouger le slider pendant un pont
// l'interrompt net (y compris un retour en arrière : on ne
// repart jamais dans le pont inverse depuis un pont).
//
// POURQUOI ÇA NE CLAQUE PAS : pick n'interrompt aucune voix en
// cours, il cesse seulement d'en produire de nouvelles. Les
// notes déjà déclenchées terminent leur enveloppe normalement.
// C'est pour ça que les couches mélodiques ont un release long
// et un peu de réverbération : leur queue sonore déborde sur le
// mouvement entrant et fabrique un vrai fondu enchaîné d'une
// demi-seconde, au lieu d'une coupe franche. Les basses et les
// percussions, elles, restent sèches — une queue dans le grave
// rendrait la bascule boueuse.
//
// Deux pièges de syntaxe, vérifiés eux aussi :
//
//  - L'ordre des arguments. C'est bien selecteur.pick([...]).
//    Écrit à l'envers (stack(...).pick(selecteur)), Strudel
//    part en boucle infinie à l'évaluation.
//
//  - slider() renvoie un Pattern, pas un nombre. Toute
//    arithmétique brute dessus (400 + ouverture * 2500) donne
//    NaN en silence, puis une erreur AudioParam à chaque note.
//    On passe donc TOUJOURS par .range(min, max).
//
// Enfin .mul(gain(volume)) et non .gain(volume) : .gain() écrase
// le gain de chaque couche (tout le monde se retrouve au même
// niveau et le mixage disparaît), alors que .mul() le multiplie.
// ============================================================

// L'ordre du tableau EST le codage de l'état : 0 STRATÉGIE,
// 1 TENSION, 2 FIN DE PARTIE, 3 pont montant, 4 pont descendant.
// pick est positionnel et ne signale rien en cas de décalage :
// une entrée insérée au milieu ferait jouer le mauvais mouvement
// en silence.
etat
  .pick([
    mouvementStrategie,
    mouvementAction,
    mouvementFinal,
    mouvementPont,
    mouvementPontDesc
  ])
  .lpf(ouverture.range(500, 12000))
  .mul(gain(volume))
