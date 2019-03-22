//
//  ViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class ViewController: NSViewController {

    @IBOutlet weak var modeLabel: NSTextField!
    @IBOutlet weak var stateLabel: NSTextField!
    @IBOutlet var textField: NSTextView!

    var contents = String();

    enum Mode: String {
        case command, insert
    }

    enum Motion: String {
        case left, right, up, down, word, word_back
    }

    var parseState: String = "" {
        didSet {
            self.stateLabel.stringValue = parseState
        }
    }

    var mode: Mode = .insert {
        didSet {
            self.modeLabel.stringValue = mode.rawValue.capitalized(with: nil)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        (NSApp.delegate as! AppDelegate).mainController = self
    }

    override func viewDidAppear() {
        self.mode = .insert
        self.textField.isEditable = true
        print("is selectable", self.textField.isSelectable)
    }

    func doMove(motion: Motion, dist: Int) {
        let selector = selectorForMoveMotion(motion: motion)
        for _ in 0..<dist {
            textField.doCommand(by: selector)
        }
    }

    func doDelete(motion: Motion, dist: Int) {

        let selector = selectorForDeleteMotion(motion: motion)
        for _ in 0..<dist {
            textField.doCommand(by: selector)
        }

        textField.doCommand(by: #selector(NSStandardKeyBindingResponding.deleteBackward(_:)))
    }


    func selectorForDeleteMotion(motion: Motion) -> Selector {
        switch motion {
        case .left:
            return #selector(NSStandardKeyBindingResponding.moveBackwardAndModifySelection(_:))
        case .right:
            return #selector(NSStandardKeyBindingResponding.moveForwardAndModifySelection(_:))
        case .word:
            return #selector(NSStandardKeyBindingResponding.moveWordForwardAndModifySelection(_:))
        case .word_back:
            return #selector(NSStandardKeyBindingResponding.moveWordBackwardAndModifySelection(_:))
        case .down:
            return #selector(NSStandardKeyBindingResponding.moveDownAndModifySelection(_:))
        case .up:
            return #selector(NSStandardKeyBindingResponding.moveUpAndModifySelection(_:))
        }
    }

    func selectorForMoveMotion(motion: Motion) -> Selector {
        switch motion {
        case .left:
            return #selector(NSStandardKeyBindingResponding.moveBackward(_:))
        case .right:
            return #selector(NSStandardKeyBindingResponding.moveForward(_:))
        case .word:
            return #selector(NSStandardKeyBindingResponding.moveWordForward(_:))
        case .word_back:
            return #selector(NSStandardKeyBindingResponding.moveWordBackward(_:))
        case .down:
            return #selector(NSStandardKeyBindingResponding.moveDown(_:))
        case .up:
            return #selector(NSStandardKeyBindingResponding.moveUp(_:))
        }
    }
}
