\version "2.24.0"
\header {
  title = "Hurrian Hymn h.6 — Hymn to Nikkal"
  subtitle = "West (1994) reconstruction"
  composer = "Anonymous (~1400 BC, Ugarit)"
  arranger = "Reconstruction: M. L. West (1994)"
  tagline = ##f
  % Source: Tablet RS 15.30 + 15.49 + 17.387
  % Tuning: nīd qablim (descending diatonic, nid qabli)
  % Genre: zaluzi (prayer to the gods)
  % Instrument: sammûm (9-string lyre)
  % Deity: Nikkal (goddess of orchards)
  % Pipeline: retro-sync / erdfa-publish / DA51 CBOR
  % License: AGPL-3.0-or-later (this engraving)
  % The underlying composition is public domain (~3400 years old)
}

\paper {
  #(set-paper-size "a4")
}

% West's reconstruction uses a descending diatonic scale on the
% 9-string sammûm.  The interval names from the tablet (qablīte,
% irbutte, etc.) are read as simultaneous pairs — dichords played
% on the lyre.  Numbers after each term indicate repetition count.
%
% Notation from Dietrich & Loretz (1975) transcription:
%   Line 1: qáb-li-te 3  ir-bu-te 1  qáb-li-te 3  ša-aḫ-ri 1
%           i-šar-te 10  ušta-ma-a-ri
%   Line 2: ti-ti-mi-šar-te 2  zi-ir-te 1  ša-aḫ-ri 2
%           ša-aš-ša-te 2  ir-bu-te 2
%
% West interprets these as dichords (string pairs) on a
% descending C–C diatonic scale: c'' b' a' g' f' e' d' c' b

% Interval mapping (West 1994):
%   qablīte    = strings 5–2 = f'–b'  (fourth)
%   irbutte    = strings 2–7 = b'–d'  (sixth)
%   šaḫri      = strings 7–5 = d'–f'  (third)
%   išarte     = strings 2–6 = b'–e'  (fourth)
%   tit.išarte = strings 3–5 = a'–f'  (third)
%   zirte      = strings 6–3 = e'–a'  (fourth)
%   šaššate    = strings 1–6 = c''–e' (sixth)

melody = \relative c'' {
  \key c \major
  \time 4/4
  \tempo "Andante" 4 = 72

  % Line 1: qablīte ×3
  <f' b>2 <f' b>2 |
  <f' b>2
  % irbutte ×1
  <b d'>2 |
  % qablīte ×3
  <f' b>2 <f' b>2 |
  <f' b>2
  % šaḫri ×1
  <d' f'>2 |
  % išarte ×10
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |
  <b e'>2 <b e'>2 |

  % Line 2: tit.išarte ×2
  <a' f'>2 <a' f'>2 |
  % zirte ×1
  <e' a'>2
  % šaḫri ×2
  <d' f'>2 |
  <d' f'>2
  % šaššate ×2
  <c'' e'>2 |
  <c'' e'>2
  % irbutte ×2
  <b d'>2 |
  <b d'>1 |
  \bar "|."
}

\score {
  \new Staff \melody
  \layout { }
  \midi { }
}
