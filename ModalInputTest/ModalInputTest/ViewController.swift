//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    override func viewDidLoad() {
        super.viewDidLoad()

        let handler = EventHandler { (val) in
            print("callback with \(val)")
        }
        for i in 0...10 {
            let r = handler.handleInput(val: UInt32(i))
//            let r = xiEventHandlerHandleInput(handler, UInt32(i))
            print("\(i): \(r)")
        }
        // Do any additional setup after loading the view.
    }

    override var representedObject: Any? {
        didSet {
        // Update the view, if already loaded.
        }
    }
}

