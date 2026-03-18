/*
 * Contains the interface definition for an Interrupt Service Routine (ISR).
 * This must be implemented by a device driver if it needs to handle interrupts.
 * The ISR is registered using the `register()` function in `intdispatcher.rs`.
 *
 * Author: Michael Schoetter, Heinrich Heine University Duesseldorf, 2022-03-10
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-01-14
 * License: GPLv3
 */

/// The Interrupt Service Routine trait.
/// This trait must be implemented by a device driver if it needs to handle interrupts.
/// The `trigger()` method is called when the corresponding interrupt occurs.
pub trait ISR {
    fn trigger(&self);
}
