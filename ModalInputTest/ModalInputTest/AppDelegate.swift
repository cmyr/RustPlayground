//
//  AppDelegate.swift
//  ModalInputTest
//
//  Created by Colin Rofls on 2019-03-18.
//  Copyright © 2019 Colin Rofls. All rights reserved.
//

import Cocoa

@NSApplicationMain
class AppDelegate: NSObject, NSApplicationDelegate {

    var mainController: ViewController?

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        // Insert code here to initialize your application
    }

    func applicationWillTerminate(_ aNotification: Notification) {
        // Insert code here to tear down your application
    }

    func handleMessage(method: String, params: [String: AnyObject]) {
        switch method {
        case "mode_change":
            let new_mode = params["mode"] as! String
            mainController?.mode = ViewController.Mode(rawValue: new_mode)!
        case "parse_state":
            let newState = params["state"] as! String
            mainController?.parseState = newState;

        case "move":
            let newMotion = params["motion"] as! String
            let motion = ViewController.Motion(rawValue: newMotion)!
            let dist = params["dist"] as! Int
            mainController?.doMove(motion: motion, dist: dist)
        case "delete":
            let newMotion = params["motion"] as! String
            let motion = ViewController.Motion(rawValue: newMotion)!
            let dist = params["dist"] as! Int
            mainController?.doDelete(motion: motion, dist: dist)

        default:
            print("unhandled method \(method)")
        }
    }
}

