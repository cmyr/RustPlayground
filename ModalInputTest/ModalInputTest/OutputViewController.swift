//
//  OutputViewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-16.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class OutputViewController: NSViewController {
    let outputFont = NSFont(name: "Menlo", size: 14.0)!

    @IBOutlet var outputTextView: NSTextView!

    override func viewDidLoad() {
        super.viewDidLoad()
        outputTextView.frame = view.frame
    }

    func appendString(_ string: String) {
        let EOFRange = NSRange(location: self.outputTextView.textStorage?.length ?? 0, length: 0)
        self.outputTextView.textStorage!.replaceCharacters(in: EOFRange, with: string)
        if EOFRange.location == 0 {
            self.outputTextView.font = self.outputFont
        }

        let bottom = CGRect(x: 2, y: outputTextView.frame.maxY + 20, width: 0, height: 0)
        outputTextView.scrollToVisible(bottom)
    }
}

extension OutputViewController: RunnerOutputHandler {
    func handleStdOut(text: String) {
        DispatchQueue.main.async {
            self.appendString(text)
        }
    }

    func handleStdErr(text: String) {
        DispatchQueue.main.async {
            self.appendString(text)
        }
    }
}
