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
    @IBOutlet weak var modeLabel: NSTextField!

    enum Mode: String {
        case command, insert
    }

    var mode: Mode = .insert {
        didSet {
            self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
            switch mode {
            case .insert:
                self.textField.isEditable = true
            case .command:
                self.textField.isEditable = false
            }
        }
    }
    override func viewDidLoad() {
        super.viewDidLoad()
        (NSApp.delegate as! AppDelegate).mainController = self
    }

    override func viewDidAppear() {
        self.mode = .insert
    }

    override func keyDown(with event: NSEvent) {
        print("VC.keyDown \(event.getAddress())")
        super.keyDown(with: event)
    }
}

