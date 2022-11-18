pub mod component;

//use gloo_net::http::Request;
use yew::{function_component, html, Callback, MouseEvent};
use yew_oauth2::{
    oauth2::{Client, Config, OAuth2},
    prelude::{Authenticated, NotAuthenticated, OAuth2Dispatcher, OAuth2Operations},
};

#[function_component(App)]
pub fn app() -> Html {
    let login = Callback::from(|_: MouseEvent| {
        OAuth2Dispatcher::<Client>::new().start_login();
    });
    let logout = Callback::from(|_: MouseEvent| {
        OAuth2Dispatcher::<Client>::new().logout();
    });

    let config = Config {
        client_id: "2qkiq3vipsjqeut8t0u4u7jkgv".into(),
        auth_url: "https://zeronote.auth.eu-north-1.amazoncognito.com/login".into(),
        token_url: "https://localhost:3000/auth/token".into(),
    };

    html!(
        <OAuth2 {config}>
            <Authenticated>
                <p>
                    <button onclick={logout}>{ "Logout" }</button>
                </p>
                //<BrowserRouter>
                //    <Switch<AppRoute> render={Switch::render(switch)}/>
                //</BrowserRouter>
            </Authenticated>
            <NotAuthenticated>
                <p>
                    <button onclick={login}>{ "Login" }</button>
                </p>
            </NotAuthenticated>
        </OAuth2>
    )
}
