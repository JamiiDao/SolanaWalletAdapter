# The Message Storage

This stores the notification messages (NotificationInfo type) which are show in the UI.

## The notification struct

The notifications are created using the `NotificationInfo` type. The internal structure of this type is as follows:

```rust,no_run
#[derive(Debug, Clone)]
pub struct NotificationInfo {
    // A unique key for each notification
    key: u32,
    // The number of seconds to show the notification
    secs: u32,
    // The message content
    message: String,
}
```

The default seconds for a notification for successful messages is shown for 2 seconds while a notification for an error is shown for 15 seconds.

### Usage

```rust,no_run
use crate::NotificationInfo;

// Create a new notification.
// It takes a generic types that ensures it can accept
// anything that implements `std::fmt::Display`.
// This default of 2 seconds for the notification to be displayed to the user. 
// This method also generates a cryptographically secure unique identifier for the message
// that allows various rust frontend frameworks to perform DOM diffing.
let notification = NotificationInfo::new("Hello");

// You can modify the default seconds from 2 to custom using
let notification = NotificationInfo::new("Hello").set_secs(100);

// To create an error notification which default to showing a notification for 15 secs use
let notification = NotificationInfo::error("User rejected the request");
```

## Global storage definition

{{#tabs }}
{{#tab name="Dioxus" }}
```rust,no_run
use dioxus::prelude::*;
use crate::GLOBAL_MESSAGE;

// Defined as a VecDeque so that new messages are pushed to the back of the queue
// and messages are simply `popped` from the front to show them to the user
pub(crate) static GLOBAL_MESSAGE: GlobalSignal<VecDeque<NotificationInfo>> =
    Signal::global(|| VecDeque::default());

#[component]
pub fn MyComponent() -> Element {
    let success = "SIWS successful!";
    GLOBAL_MESSAGE
		.write()
    	// use `push_back` method to push messages to the queue
		.push_back(NotificationInfo::new(success));
    
    
    let myerror = "User rejected the request!";
    GLOBAL_MESSAGE
        // use `push_back` method to push messages to the queue
		.write()
		.push_back(NotificationInfo::new(myerror));
}
```
{{#endtab }}

{{#tab name="Sycamore" }}
```rust,no_run
use sycamore::prelude::*;

// Defined as a VecDeque so that new messages are pushed to the back of the queue
// and messages are simply `popped` from the front to show them to the user
pub type GlobalMessage = VecDeque<NotificationInfo>;

#[component]
fn App() -> View {
    // `GlobalMessage` is instantiated and wrapped in a `provide_context` so that it is
    // accessible in a global context
	provide_context(create_signal(GlobalMessage::default()));
}

// `GlobalMessage` now available in another module 
mod another_scope {
    use sycamore::prelude::*;
    use crate::{GlobalMessage, NotificationInfo};
    
    #[component]
    pub fn ConnectWalletModalModal() -> View {
        // Import the `GlobalMessage` from the global context
        let global_message = use_context::<Signal<GlobalMessage>>();
        
        let error = "User rejected the request";
        // Use the `update` method on a Sycamore signal to gain mutable access to the store
        // and `.push_back()` method to insert the message to the message store
        global_message.update(|store| store.push_back(NotificationInfo::error(error)));
    }
}
```
{{#endtab }}

{{#tab name="Yew" }}
```rust,no_run
use yew::prelude::*;
use crate::GlobalAppInfo;

// Defined as a VecDeque so that new messages are pushed to the back of the queue
// and messages are simply `popped` from the front to show them to the user.

#[function_component(App)]
pub fn app() -> Html {
    let adapter = WalletAdapter::init().unwrap();

    // The message store is part of the `GlobalAppInfo` type
    let init_state = GlobalAppInfo::new(adapter);

       html! {
        <ContextProvider<GlobalAppState> context={global_state.clone()}>
			// Any child component defined here has access to the `GlobalAppInfo`
           // which contains the messages store.
           <MyComponent/>
        </ContextProvider<GlobalAppState>>
    }
}

mod another_scope {
    use yew::prelude::*;
    use crate::{GlobalAppState, GlobalAction, NotificationInfo};
    
    #[function_component]
    pub fn MyComponent() -> Html {        
        let global_state =
            use_context::<GlobalAppState>().expect("no global ctx `GlobalAppState` found");
        let error = "User rejected the request";
        
        // Yew 0.21 uses message passing to update state
        global_state.dispatch(GlobalAction::Message(NotificationInfo::new(error)))
    }
}
```
{{#endtab }}
{{#endtabs }}
