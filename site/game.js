import init, {GameState, start_state, SelectionSuccess, GameFailiure, TranscodingError} from './pkg/nyt_connections.js';



async function run() {
    const url = "./pkg/nyt_connections_bg.wasm";
    const FetchOptions = {
        headers:{
            "Content-Type": "application/wasm",
        }
    };
    const wasm = fetch(url, FetchOptions);

    const module = await WebAssembly.compileStreaming(wasm);

    await init(module);

    //const module = WebAssembly.compileStreaming(fetch('./pkg/nyt_connections_bg.wasm'));
    //await init();
    //await init(module);
}



function main(state){

    const initializeCards = (card, card_key) => {
        card.addEventListener("click", () => {
            state.select(card,card_key)
            //CardClick(card, index);
        });
        //card.addEventListener("mouseover", () => {state.select(index)});
    }

    function shake_selection(){
        const cards = Array.from(document.getElementsByClassName("card"));
        const indices = Array.from(state.get_selection_indices());
        indices.forEach(async (i) => {
            cards[i].style.animation = "shake linear .25s";
	    await new Promise(r => setTimeout(r, 250));
            cards[i].style.animation = "";
        });
    }

    async function jump_selection() {
        const cards = Array.from(document.getElementsByClassName("card"));
        const indices = Array.from(state.get_selection_indices());
        for(const i of indices){
            cards[i].style.animation = "jump linear .25s";
	    await new Promise(r => setTimeout(r, 250));
            cards[i].style.animation = "";

        }

    }

    function one_away(){
        const elem = document.getElementById("away");
        animate(elem);
    }

    function already_guessed(){
        const elem = document.getElementById("already");
        animate(elem);
    }

    async function animate(element){
        element.show();
	await new Promise(r => setTimeout(r, 2000));
	element.animate([{opacity:1,display:"block"},{opacity:0,display:"none"}],400);
	await new Promise(r => setTimeout(r, 350));
        element.close();
    }

    function display_won(){
	document.getElementById("endscreen").showModal();
        //document.getElementById("overlay-container").classList.add("enabled");
        document.getElementById("win").classList.add("enabled");
        show_end_buttons();
    }

    function show_end_buttons(){
        document.getElementById("again").classList.add("enabled");
        document.getElementById("share").classList.add("enabled");
        document.getElementById("back").classList.add("enabled");
        document.getElementById("edit-me").classList.add("enabled");
    }

    function display_lost(){
	document.getElementById("endscreen").showModal();
        document.getElementById("lose").classList.add("enabled");
        show_end_buttons();
    }

    function hide_overlay(){

	document.getElementById("endscreen").close();
        //document.getElementById("overlay-container").classList.remove("enabled");
        document.getElementById("win").classList.remove("enabled");
        document.getElementById("lose").classList.remove("enabled");
        document.getElementById("again").classList.remove("enabled");
        document.getElementById("share").classList.remove("enabled");
        document.getElementById("back").classList.remove("enabled");
    }

    const init_buttons = () => {
        const submit = document.getElementById("submit");
        submit.addEventListener("click", async () => {
	    const classList = submit.classList;
	    if(classList.contains("hidden")){
		    return
	    }
	    classList.add("hidden");
            try{
                await jump_selection();
                if(state.check_selection() == SelectionSuccess.Won){
                    display_won();
                }
            }catch (exceptionVar){
		    //console.log(GameFailiure[exceptionVar]);
                switch (exceptionVar){
                    case GameFailiure.Mismatch: // MISMATCH
                        shake_selection();
                        break;
                    case GameFailiure.NotEnough: // NOT ENOUGH 
                        shake_selection();
                        break;
                    case GameFailiure.OneAway: //One Away
                        shake_selection();
                        one_away();
                        break;
                    case GameFailiure.Lost:
                        shake_selection();
                        display_lost();
                        break;
                    case GameFailiure.AlreadyTried:
                        already_guessed();
                }
            }finally{
	    	submit.classList.remove("hidden");
	    }
        });

        const shuffle = document.getElementById("shuffle");
        shuffle.addEventListener("click", async () => {
	    const elems = Array.from(document.getElementsByClassName("card"));
	    elems.forEach((elem) => {
                elem.classList.toggle("shuffling")
                elem.classList.remove("selected")
            });

	    await new Promise(r => setTimeout(r, 175));
            state.shuffle();
	    elems.forEach((elem) => {
                elem.classList.toggle("shuffling")
            });
        });

        const deselect = document.getElementById("deselect");

        deselect.addEventListener("click", () => {
	    if(deselect.classList.contains("hidden")){
		    return
	    }
            state.clear_selection();
        });

        const try_again = document.getElementById("again")
        try_again.addEventListener("click", () =>{
            state.start_over();
	    setup_cards();
            hide_overlay();
        });

        const share = document.getElementById("share")
        share.addEventListener("click", async () =>{
	    const code = state.puzzle_code();
	    const url = new URL(document.location.href);
		// is this secure,,
	    url.searchParams.set("game",code);
	    await window.navigator.clipboard.writeText(url.href);
	    const copied = document.getElementById("copied");
	    animate(copied);
	    //state.clipboard_copied();
        });

        const back = document.getElementById("back")
        back.addEventListener("click", () =>{   
	    const url = new URL("../",document.location.href);
            self.window.location.assign(url);
        });

        const edit_me = document.getElementById("edit-me")
        edit_me.addEventListener("click", () =>{
	    const code = state.puzzle_code();
	    console.log(code);
	    const url = new URL("../",document.location.href);
	    url.searchParams.delete("game");
	    url.searchParams.set("puzzle",code);
	    console.log(url);
            self.window.location.assign(url);
        });
    }

    function setup_cards(){
	const game_board = document.getElementById("board");
	const cards = Array.from(game_board.children);
	cards.forEach(initializeCards);
	state.render_cards();
    }

    setup_cards();
    init_buttons();
    console.log(state.puzzle_code());
}

function default_main(){
    const state = start_state();
    main(state)
}

function main_with_code(code){
    try{
        let state = GameState.from_code(code);
        main(state)
    }catch(e){
        let error;
        switch (e){
            case TranscodingError.Base64:
                error = "base64";
                break;
            case TranscodingError.Gzip:
                error = "gzip";
                break;
            case TranscodingError.Cbor:
                error = "postcard";
                break;
        }
        console.log("error",e, error);
    }
}


function entry_point(){
    let url = new URL(document.URL);
    const game_code = url.searchParams.get("game");
    if(game_code != null){
        main_with_code(game_code)
    }else{
        default_main();
    }
}

addEventListener("load", (_) => {
    document.documentElement.removeAttribute("hidden");
});

await run();
entry_point();

