//
//  MainPlaygroundVewController.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-04-15.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

let OUTPUT_TOOLBAR_ITEM_TAG = 10;

class MainPlaygroundViewController: NSSplitViewController {

    var outputViewController: OutputViewController {
        return splitViewItems[1].viewController as! OutputViewController
    }

    var editViewController: EditViewController {
        return splitViewItems[0].viewController as! EditViewController
    }

    lazy var toggleOutputToolbarButton: NSButton = {
        let toolbarItem = view.window?.toolbar?.items.first {
            $0.tag == OUTPUT_TOOLBAR_ITEM_TAG
        }
        return toolbarItem!.view as! NSButton
        }()

    override func viewDidLoad() {
        super.viewDidLoad()
    }

    override func viewDidAppear() {
        super.viewDidAppear()
        let initSplitHeight = max(200, view.frame.height / 3).rounded(.down);
        splitView.setPosition(view.frame.height - initSplitHeight, ofDividerAt: 0)
        splitViewItems[1].isCollapsed = true
    }

    var outputViewIsVisible: Bool = false {
        didSet {
            let isVisible = outputViewIsVisible
            toggleOutputToolbarButton.highlight(true)
            NSAnimationContext.runAnimationGroup({ (context) in
                context.allowsImplicitAnimation = true
                context.duration = 0.25
                self.splitViewItems[1].isCollapsed = !isVisible
                self.splitView.layoutSubtreeIfNeeded()
            }, completionHandler: {
                let newState: NSControl.StateValue = isVisible ? .on : .off
                self.toggleOutputToolbarButton.highlight(false)
                self.toggleOutputToolbarButton.state = newState
            })
        }
    }

    @IBAction func toggleOutputView(_ sender: NSButton?) {
        outputViewIsVisible = !outputViewIsVisible
    }

    @IBAction func build(_ sender: Any?) {
        let workDirectory = "/Users/rofls/dev/hacking/macos_rustplay_test"
        let fileName = "playground_test.rs"
        //        let execName = "playground_test"
        let magicNumber: UInt32 = 6942069

        if !outputViewIsVisible {
             outputViewIsVisible = true
        }

        let document = (NSApp.delegate as! AppDelegate).core.getLine(magicNumber)!.text

        let directory = URL(fileURLWithPath: workDirectory)
        let fileUrl = directory.appendingPathComponent(fileName, isDirectory: false)

        try! document.write(to: fileUrl, atomically: true, encoding: .utf8)
        let scriptURL = BundleResources.buildScriptURL
        let runner = Runner(scriptPath: scriptURL, fileName: fileName)
        if runner.compile(handler: outputViewController) {
            runner.run(handler: outputViewController)
        }
    }
}
