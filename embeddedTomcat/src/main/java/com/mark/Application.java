package com.mark;

import java.io.File;
import java.io.IOException;
import java.net.URISyntaxException;
import java.nio.file.Files;
import java.nio.file.Path;
import org.apache.catalina.LifecycleException;
import org.apache.catalina.WebResourceRoot;
import org.apache.catalina.core.StandardContext;
import org.apache.catalina.startup.Tomcat;
import org.apache.catalina.webresources.DirResourceSet;
import org.apache.catalina.webresources.JarResourceSet;
import org.apache.catalina.webresources.StandardRoot;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Application {

    private static final Logger logger = LoggerFactory.getLogger(Application.class);

    public static void main(String[] main) throws URISyntaxException, IOException, LifecycleException {
        File root = getRootFolder();
        Path tempPath = Files.createTempDirectory("temp");

        Tomcat tomcat = new Tomcat();
        tomcat.setBaseDir(tempPath.toString());
        tomcat.setPort(8080);
        tomcat.getConnector();


        StandardContext ctx = (StandardContext) tomcat.addWebapp("", new File(root.getAbsolutePath()).getAbsolutePath());

        // Load Servlet config into Servlet Context for accessibility
        // ctx.getServletContext().setAttribute("aKey", "aValue");

        // Check if running within a jar and use appropriate resource set object
        String runningUriPath = Application.class.getProtectionDomain().getCodeSource().getLocation().toURI().getPath();
        WebResourceRoot resources = new StandardRoot(ctx);
        resources.addPreResources(runningUriPath.toUpperCase().endsWith(".JAR")
                ? new JarResourceSet(resources, "/WEB-INF/classes", new File(runningUriPath).getAbsolutePath(), "/")
                : new DirResourceSet(resources, "/WEB-INF/classes", new File(runningUriPath).getAbsolutePath(), "/"));
        ctx.setResources(resources);

        logger.info("Application root: " + root.getAbsolutePath());
        logger.info("Listening port: 8080");

        tomcat.start();
        tomcat.getServer().await();
    }

    private static File getRootFolder() throws URISyntaxException {
        File root;
        String runningJarPath = Application.class.getProtectionDomain().getCodeSource().getLocation().toURI().getPath().replaceAll("\\\\", "/");
        int lastIndexOf = runningJarPath.lastIndexOf("/target/");
        if (lastIndexOf < 0) {
            root = new File("");
        } else {
            root = new File(runningJarPath.substring(0, lastIndexOf));
        }
        return root;
    }
}


