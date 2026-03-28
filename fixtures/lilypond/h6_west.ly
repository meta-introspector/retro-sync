\version "2.24.0"
\header {
  title = "Hurrian Hymn h.6 — Hymn to Nikkal"
  subtitle = "West (1994) reconstruction from tablet RS 15.30"
  composer = "Urẖiya (composer) · Ammurabi (scribe) · ~1400 BC, Ugarit"
  arranger = "Reconstruction: M. L. West (1994)"
  piece = "zaluzi (prayer to Nikkal, goddess of orchards)"
  tagline = \markup {
    \column {
      \line { "DA51 · Cl(15,0,0) · 6-layer stego · shard 1/71" }
      \line { "𒀸𒌑𒄴𒊑  nīš tuḫrim · Tablet RS 15.30 · Yazılıkaya" }
    }
  }
  % Tuning: nīd qablim (descending diatonic C–C on 9-string sammûm)
  % Strings: 1=c'' 2=b' 3=a' 4=g' 5=f' 6=e' 7=d' 8=c' 9=b
  % Genre: zaluzi (prayer to Nikkal, goddess of orchards)
  % Scribe: Ammurabi
}

\paper { #(set-paper-size "a4") }

% West's descending diatonic scale on the sammûm:
%   String: 1    2    3    4    5    6    7    8    9
%   Pitch:  c''  b'   a'   g'   f'   e'   d'   c'   b
%
% Interval mapping (string pairs → dichords):
%   qablītum       5–2  = f' + b'   (fourth)
%   irbutte/rebûttum 2–7 = b' + d'  (sixth)
%   šaḫri/šērum    7–5  = d' + f'   (third)
%   išartum        2–6  = b' + e'   (fourth)
%   tit.išartim    3–5  = a' + f'   (third)
%   zirte          6–3  = e' + a'   (fourth)
%   šaššatum       1–6  = c'' + e'  (sixth)
%
% Tablet notation (Dietrich & Loretz 1975):
%   Line 1: qablīte ×3, irbutte ×1, qablīte ×3, šaḫri ×1,
%           išarte ×10, uštamari (colophon)
%   Line 2: tit.išarte ×2, zirte ×1, šaḫri ×2,
%           šaššate ×2, irbutte ×2

melody = \relative c' {
  \key c \major
  \time 4/4
  \tempo "Andante" 4 = 72

  % Line 1: qablīte ×3 (f'+b')
  <f' b'>2^\markup { \small "qablīte" } <f' b'>2 |
  <f' b'>2
  % irbutte ×1 (b'+d')
  <b' d'>2^\markup { \small "irbutte" } |
  % qablīte ×3 (f'+b')
  <f' b'>2^\markup { \small "qablīte" } <f' b'>2 |
  <f' b'>2
  % šaḫri ×1 (d'+f')
  <d' f'>2^\markup { \small "šaḫri" } |

  % išarte ×10 (b'+e') — the long central section
  <b e'>2^\markup { \small "išarte ×10" } <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |

  % Line 2: tit.išarte ×2 (a'+f')
  <a' f'>2^\markup { \small "tit.išarte" } <a' f'>2 |
  % zirte ×1 (e'+a')
  <e' a'>2^\markup { \small "zirte" }
  % šaḫri ×2 (d'+f')
  <d' f'>2^\markup { \small "šaḫri" } |
  <d' f'>2
  % šaššate ×2 (c''+e')
  <c'' e'>2^\markup { \small "šaššate" } |
  <c'' e'>2
  % irbutte ×2 (b'+d')
  <b' d'>2^\markup { \small "irbutte" } |
  <b' d'>1 |

  \bar "|."
}

\score {
  \new Staff \with {
    instrumentName = "Sammûm"
  } \melody
  \layout { }
  \midi { }
}
