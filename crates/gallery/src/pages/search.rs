//! The match behind a command palette, shown by running it.
//!
//! The lists below are filled from `rank()` itself rather than written out, which
//! is the whole point of the page: an expected output copied into a page proves
//! nothing, and a ranking rule is exactly the kind of thing that is subtly wrong in
//! a way only a reader notices. Here the running app *is* the evidence.
//!
//! Three lists, one candidate set, three queries — chosen because they are the
//! cases that decided the weights:
//!
//! - **no query** matches everything in the order given, which is what a palette
//!   shows before anything is typed;
//! - **`nt`** must lead with `New Terminal` over `environment.rs`-shaped
//!   accidents, so a word start has to be able to dominate;
//! - **`term`** must lead with `New Terminal` over `The remote endpoint`, which
//!   also matches two word starts but scatters — so a contiguous run has to be
//!   able to dominate a word start. Those two point in opposite directions, which
//!   is why this is a weighted score and not a comparison order.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*
    use mod.mp.*
    use mod.mpc.tokens.*
    use mod.mpc.type.*

    let Caption = Label{
        width: Fit, height: Fit
        draw_text +: {text_style: caption, color: text_muted}
    }
    let Section = View{
        width: Fill, height: Fit, flow: Down, spacing: 8
    }

    mod.gallery.pages.search = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Search"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A match is a case-insensitive subsequence, so three keystrokes find a command — gtf finds Go to File. That is the part a palette cannot work without. The ranking is the part that is easy to get subtly wrong, so the two preferences that decide it point in opposite directions and neither can be a comparison order. The first version of the file tried one and was wrong about term; the lists below are why."
        }

        Section{
            Caption{ text: "The field — a magnifier that leads, so the eye finds the field by its icon, and a shortcut that trails, so it is read after the query rather than before it" }
            mod.mp.MpSearch{
                width: 460
            }
        }

        Section{
            Caption{ text: "1 · No query matches everything, in the order given. A palette's first frame" }
            search_all := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "2 · “nt” leads with New Terminal — a word start has to be able to dominate, or a palette answers with substrings of words the reader never typed" }
            search_nt := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "3 · “term” also leads with New Terminal, now over The remote endpoint — which matches two word starts (T of The, r of remote) but scatters. A contiguous run has to be able to dominate a word start, and that is the opposite of 2" }
            search_term := mod.mp.MpMenu{}
        }

        Section{
            Caption{ text: "4 · But two word starts still beat one contiguous run — “or” finds Off Road, not Word. This case fixes the ratio between the two weights: it is the one a run-first order gets wrong" }
            search_or := mod.mp.MpMenu{}
        }
    }
}
