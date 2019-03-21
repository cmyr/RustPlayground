//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {
    @IBOutlet weak var textField: NSTextField!

    override func viewDidLoad() {
        super.viewDidLoad()
    }

    override func viewDidAppear() {
        self.view.window?.makeFirstResponder(self.textField)
    }

    override func keyDown(with event: NSEvent) {
        print("VC.keyDown \(event.getAddress())")
        super.keyDown(with: event)
    }
}

