# Demo

Open `/?demo=1`, `/demo`, or click **Try it with sample data** on the home page. It loads an isolated sample review packet for “Add organization-level retention controls,” with a contract change, a migration, test evidence, and two required-owner checks.

The demo does not call the packet API and sends no data off-origin. Its temporary UI state uses the `demo:diff-gate` session-storage namespace. **Reset demo** restores the shipped sample. **Start for real** or navigation to another product page leaves demo mode and discards its state.
