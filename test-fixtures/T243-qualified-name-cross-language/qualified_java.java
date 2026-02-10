// Java qualified names using .
package com.example.myapp;

import java.util.ArrayList;
import com.example.services.UserService;

public class Application {
    public static void main(String[] args) {
        ArrayList<String> list = new ArrayList<>();
        UserService service = new UserService();
    }
}
