//
//  MainPlaygroundVewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

class MainPlaygroundViewController: NSSplitViewController {

    override func viewDidLoad() {
        super.viewDidLoad()
        splitViewItems[1].isCollapsed = true
    }

    var outputViewIsVisible: Bool = false {
        didSet {
            guard outputViewIsVisible != oldValue else { return }
            let isVisible = outputViewIsVisible
            if isVisible {
                self.splitViewItems[1].isCollapsed = false
            }
            // adapted from https://stackoverflow.com/questions/26664759/how-to-collapse-an-nssplitview-pane-with-animation-while-using-auto-layout
            NSAnimationContext.runAnimationGroup({ (context) in
                context.allowsImplicitAnimation = true
                context.duration = 0.25
                if isVisible {
                    self.splitView.setPosition(self.view.frame.height - 140, ofDividerAt: 0)
                } else {
                    self.splitView.setPosition(self.view.frame.height, ofDividerAt: 0)
                }
                self.splitView.layoutSubtreeIfNeeded()
            }) {
                if !isVisible {
                    self.splitViewItems[1].isCollapsed = true
                }
            }
        }
    }


    @IBAction func toggleOutputView(_ sender: NSButton?) {
//        let button = sender as! NSButton
        outputViewIsVisible = sender!.state == .on
    }

    @IBAction func build(_ sender: Any?) {
        let workDirectory = "/Users/rofls/dev/hacking/macos_rustplay_test"
        let fileName = "playground_test.rs"
        //        let execName = "playground_test"
        let magicNumber: UInt32 = 6942069

        let document = (NSApp.delegate as! AppDelegate).core.getLine(magicNumber)!.text

        let directory = URL(fileURLWithPath: workDirectory)
        let fileUrl = directory.appendingPathComponent(fileName, isDirectory: false)

        try! document.write(to: fileUrl, atomically: true, encoding: .utf8)
        let scriptURL = BundleResources.buildScriptURL
        let runner = Runner(scriptPath: scriptURL, fileName: fileName)
        if runner.compile() {
            runner.run()
        }
    }
}
