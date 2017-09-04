use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path};
use std::os::unix::fs::PermissionsExt;
use std::{env,process};

struct DirectoryInfo{
    path:String,
    mode:u32,
}

struct DirectoryWalker{
    dir_list:Vec<DirectoryInfo>,
    current_dir_mode:u32,
    root_dir:String,
    root_dir_mode:u32,
}

//ディレクトリ走査を行うための実装
impl DirectoryWalker{
    fn new(root_dir:&str)->io::Result<DirectoryWalker>{
        let mut dir_list = Vec::<DirectoryInfo>::new();
        let root_dir_mode = Path::new(&root_dir).metadata()?.permissions().mode();
        // アクセス権限は、上位ディレクトリのものを継承する。
        dir_list.push(DirectoryInfo{path:root_dir.to_owned(),mode:root_dir_mode});
        let dw = DirectoryWalker{
            dir_list:dir_list,
            current_dir_mode:0,
            root_dir:root_dir.to_owned(),
            root_dir_mode:root_dir_mode,
        };
        Ok(dw)
    }
    
    fn dir_walk(&mut self,dir:&DirectoryInfo,cb_file:fn(walker:&DirectoryWalker,entry:DirEntry,parent:&DirectoryInfo)->io::Result<()>) -> io::Result<()> {
        let parent=dir;
        let dir=Path::new(&dir.path);
        self.current_dir_mode = dir.metadata()?.permissions().mode();
        for entry in fs::read_dir(dir)?{
            let entry = entry?;
            if entry.path().is_dir() {
                let mode = entry.path().metadata()?.permissions().mode();
                // ファイルの可視性を仮想的に実装する。
                // パーミッションの継承を行う。
                self.dir_list.push(DirectoryInfo{path:entry.path().into_os_string().into_string().unwrap(),mode:parent.mode&mode});
            }else{
                // self : 現在のディレクトリのパーミッションと検索開始ディレクトリのパーミッション
                // entry : ファイルの情報（パスとファイルシステムから取得できるパーミッション）
                // parent : 親ディレクトリ（パスと論理パーミッション（継承されたもの））
                cb_file(&self,entry,parent)?;
            }
        }
        Ok(())
    }
}

fn callback(walker:&DirectoryWalker,entry:DirEntry,parent:&DirectoryInfo) -> io::Result<()> {
    Ok(())
}

fn main() {
    let prog:String = env::args().next().unwrap_or("".to_owned());
    let args:Vec<String> = env::args().skip(1).collect();
    if args.len()<1{
        println!("{} <path>",prog);
        process::exit(1);
    }
    let audit_path = args[0].to_owned();
    let mut walker = match DirectoryWalker::new(&*audit_path){
        Ok(walker)=>walker,
        Err(e)=>{
            println!("walker generate fail : {}",e);
            return;
        }
    };

    loop{
        let dir=walker.dir_list.pop().unwrap();
        match walker.dir_walk(&dir,callback){
            Ok(_)=>{},
            Err(e)=>{println!("{}:{}",dir.path,e);}
        };
        if walker.dir_list.len()==0{break;}
    }
}
