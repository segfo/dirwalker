use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path};
use std::os::unix::fs::PermissionsExt;
use std::{env,process};

struct DirectoryWalker{
    dir_list:Vec<String>,
    current_dir_mode:u32,
    root_dir:String,
    root_dir_mode:u32,
}

impl DirectoryWalker{
    fn new(root_dir:&str)->io::Result<DirectoryWalker>{
        let mut dir_list = Vec::<String>::new();
        dir_list.push(root_dir.to_owned());
        let root_dir_mode = Path::new(&root_dir).metadata()?.permissions().mode();
        let dw = DirectoryWalker{
            dir_list:dir_list,
            current_dir_mode:0,
            root_dir:root_dir.to_owned(),
            root_dir_mode:root_dir_mode,
        };
        Ok(dw)
    }

    fn dir_walk(&mut self,dir:&String,cb_file:fn(walker:&DirectoryWalker,parent:&String,entry:DirEntry)->io::Result<()>) -> io::Result<()> {
        let parent=dir;
        let dir=Path::new(&dir);
        self.current_dir_mode=dir.metadata()?.permissions().mode();
        for entry in fs::read_dir(dir)?{
            let entry = entry?;
            if entry.path().is_dir() {
                self.dir_list.push(entry.path().into_os_string().into_string().unwrap());
            }else{
                cb_file(&self,parent,entry)?;
            }
        }
        Ok(())
    }
}

fn callback(walker:&DirectoryWalker,parent:&String,entry: DirEntry) -> io::Result<()> {
    let md=entry.path().metadata()?.permissions().mode();
    println!("{:o}:{}:{:o}:{}",walker.current_dir_mode,parent,md,entry.path().to_string_lossy());
    Ok(())
}

fn main() {
    let prog:String = env::args().next().unwrap_or("".to_owned());
    let args:Vec<String> = env::args().skip(1).collect();
    if args.len()<1{
        println!("{} <path>",prog);
        process::exit(1);
    }

    let path = args[0].to_owned();
    let mut walker = match DirectoryWalker::new(&*path){
        Ok(walker)=>walker,
        Err(e)=>{
            println!("walker generate fail : {}",e);
            return;
        }
    };

    loop{
        let dir=walker.dir_list.pop().unwrap();
        walker.dir_walk(&dir,callback).unwrap();
        if walker.dir_list.len()==0{break;}
    }
}
