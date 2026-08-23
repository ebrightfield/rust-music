\version "2.24.4"
\header { tagline = ##f }
\paper { line-width = 185\mm indent = 0 ragged-right = ##f }
call = {
  \key g \major \time 4/4
  d'8\3-1 e'\3-3 g'\2-2 a'\2-4 g'4\2-2 r4 |
  e'8\3-3 d'\3-1 b\3-4 d'4\3-1 r4 r8 \bar "||"
}
answer = {
  \key g \major \time 4/4
  \override NoteHead.style = #'slash
  \hideNotes r1 | r1 \unHideNotes \bar "|."
}
\score { <<
  \new Staff { \clef "treble_8" \override StringNumber.stencil = ##f
    \mark \markup { \small \bold "CALL (play as written)" } \call
    \mark \markup { \small \bold "ANSWER (yours — 2 bars)" } \answer }
  \new TabStaff { \call \answer }
>> \layout {} }
