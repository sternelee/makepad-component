//! The calendar, and the arithmetic that is the hard part of it.
//!
//! Two grids, because the only configuration a calendar has is which day a week
//! starts on and the two answers have to be visible side by side — a Monday-first
//! grid that is a Sunday-first grid with the wrong column headings is a fault nobody
//! catches by looking at one of them.
//!
//! "Today" here is the **real** date: the app reads the clock and converts it with
//! `date::date_from_days_since_epoch`, and the library never touches a clock. That
//! split is the point — a widget that called `SystemTime::now()` itself would be
//! untestable and would disagree with the app's own idea of today across midnight.
//!
//! The page prints the date it computed, because a ring drawn around "today" is only
//! evidence if the reader can check which day it landed on.

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

    mod.gallery.pages.calendar = View{
        width: Fill
        height: Fit
        flow: Down
        spacing: 20

        Label{
            width: Fit, height: Fit
            draw_text +: {text_style: title2, color: text}
            text: "Calendar"
        }
        Label{
            width: Fill, height: Fit
            draw_text +: {text_style: footnote, color: text_muted}
            text: "A month grid is a header, seven letters and forty-two numbers. The numbers are the part that can be wrong, and a wrong calendar is wrong on a small fraction of dates — the worst possible failure rate, because it survives every look-at-it test and then puts an appointment on the wrong day. Two of the ways it goes wrong are invisible in a screenshot of one month: the leap rule (2000 is a leap year, 1900 and 2100 are not) and the weekday of a date, which needs a real algorithm. The weekday here is checked not against an anchor but against a second, independent computation that accumulates one day at a time, so an error in one has to be an error in the other."
        }

        Section{
            Caption{ text: "Sunday-first, the default. The grid is always six rows — a month needs between four and six, and a calendar that changes height moves its own navigation arrows, so the reader's pointer lands on a different control in March than it did in February. The ring is today; the filled plate is the selection; the two arrows page by month" }
            View{
                width: Fill, height: Fit, flow: Right, spacing: 20
                align: Align{x: 0.0, y: 0.0}
                cal_sunday := mod.mp.MpDate{}
                View{
                    width: Fill, height: Fit, flow: Down, spacing: 10
                    cal_readout := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_muted}
                        text: "(selection)"
                    }
                    cal_today_note := Label{
                        width: Fill, height: Fit
                        draw_text +: {text_style: caption, color: text_faint}
                        text: "(today)"
                    }
                    Caption{ text: "Today is read from the clock by the app and converted with date_from_days_since_epoch, whose test is a round trip against days_since_epoch over two hundred years. The library never reads a clock — a widget that called SystemTime::now() itself would be untestable, and would disagree with the app's own idea of today across a midnight boundary." }
                }
            }
        }

        Section{
            Caption{ text: "Monday-first. The same week, started one column later — so Sunday is the last column. Its headings are a rotation of the same seven days rather than a second array, because a second array is a second thing to get wrong when a weekday is renamed, and the test that catches a bad rotation is counting the two Ss and the two Ts" }
            cal_monday := mod.mp.MpDate{
                week_start: mod.mp.WeekStart.Monday
            }
        }

        Section{
            Caption{ text: "A month with the worst-case grid: 31 days starting on the last column of the first row, so it needs all six. Nothing about the widget's height changes" }
            mod.mp.MpDate{
                cal_worst := mod.mp.MpDate{}
            }
        }
    }
}
