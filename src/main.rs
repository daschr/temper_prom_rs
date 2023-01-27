mod temper;

use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::process;
use std::env;

use tide::Request;

const UPDATE_INTERVAL:time::Duration = time::Duration::from_secs(3);

#[derive(Clone)]
struct State {
    temp: Arc<Mutex<Vec<f32>>>
}

#[async_std::main]
async fn main() ->  tide::Result<()> {
    let temper_instance=temper::Temper::new().unwrap();

    let mut sticks=temper_instance.get_sticks();

    for i in 0..sticks.len(){
        if let Err(e)=sticks[i].init(){
            eprintln!("Error: {}", e);
            process::exit(1);
        }else{
           println!("{}", sticks[i].get_temp().unwrap());
        }
    }
    
    let state=State{temp: Arc::new(Mutex::new(Vec::with_capacity(sticks.len())))};


    let mut app = tide::with_state(state.clone());

    app.at("/").get(|req: Request<State>| async move {
        let p=req.state().clone();
        let mut msg=String::new();
        
        {
            let temps=p.temp.lock().unwrap();
            for i in 0..temps.len() {
                msg.push_str(&format!("temp{}: {}", i, temps[i]));
            }
        }

        Ok(msg)
    });

    thread::spawn(move || {
                loop {        
                    {
                        let p=state.temp.clone();
                        let mut temp = p.lock().unwrap();
                        for i in 0..sticks.len() {
                            temp[i]=sticks[i].get_temp().unwrap();
                        }
                    }
                    thread::sleep(UPDATE_INTERVAL);
                }
            });

    let listen_addr=format!("0.0.0.0:{}", { if env::args().len()>1 { env::args().nth(2).unwrap() } else { String::from("8777") }});
    app.listen(listen_addr).await?;
    
    Ok(())
}
