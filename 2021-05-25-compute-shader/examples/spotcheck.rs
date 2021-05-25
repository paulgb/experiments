use clap::Clap;

#[derive(Clap)]
struct Opts {
    real: f32,
    imag: f32,
    #[clap(default_value="10")]
    iterations: u32,
}

fn main() {
    let opts = Opts::parse();

    let cx = opts.real;
    let cy = opts.imag;

    let mut x: f32 = 0.;
    let mut y: f32 = 0.;
    let mut final_iters: u32 = 0;

    for i in 0..opts.iterations {
        final_iters = i;
        if x*x + y*y > 4. {
            break;
        }
        
        let xtemp = (x*x) - (y*y) + cx;
        y = 2. * x * y + cy;
        x = xtemp;
    }

    println!("{}", final_iters);
    println!("{} {}", x, y);
}