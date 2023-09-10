/*
    Copyright 2023 github.com/programaquatic

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use serde::{Deserialize};
use std::fs;
    
pub fn read_json<T>(file: String) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>
{
    // let mut holes = Vec::new();
    // holes.push( HoleAndLocation { position: RelPosition::Bottom, x: 10, y: 10, diameter: 6 });
    // holes.push( HoleAndLocation { position: RelPosition::Bottom, x: 20, y: 10, diameter: 6 });

    // let mut shaftline = Vec::new();
    // shaftline.push( Position2D { x: 0, y: 15 } );
    // shaftline.push( Position2D { x: 25, y: 15 } );
    // shaftline.push( Position2D { x: 35, y: 0 });
 
    // let intank = Tank{
    //     tank: TankDimensions {
    //         width: 150,
    //         depth: 80,
    //         height: 70,
    //         glass: 15,
    //     },
    //     overflow: OverFlowData {
    //         drill: holes,
    //         shaft: shaftline,
    //     }
    // };
    
    // let ostr = serde_json::to_string_pretty(&intank).unwrap();
    // println!("{}",ostr);
    let cfg_content_iter = fs::read_to_string(file).expect("Failed to open config config file");
    let rem_lines = cfg_content_iter.lines().filter(|l| ! l.trim_start().starts_with("//") );
    
    let mut cfg_json = String::from("");
    rem_lines.for_each(|l| cfg_json.push_str(l) );
    
    let data: T = serde_json::from_str(&cfg_json).expect("Format error in json");
    Ok(data)
}
