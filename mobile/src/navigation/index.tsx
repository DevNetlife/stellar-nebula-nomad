import React, { useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { RootStackParamList, AuthStackParamList, MainTabsParamList } from '../types';
import { useAppStore } from '../store';
import { LockManager } from '../auth/LockManager';
import { HomeScreen } from '../screens/HomeScreen';
import { WalletScreen } from '../screens/WalletScreen';
import { SettingsScreen } from '../screens/SettingsScreen';
import BiometricSetupScreen from '../screens/BiometricSetup';
import LockScreen from '../screens/LockScreen';

const RootStack = createNativeStackNavigator<RootStackParamList>();
const AuthStack = createNativeStackNavigator<AuthStackParamList>();
const MainTabs = createBottomTabNavigator<MainTabsParamList>();

export const AuthNavigator: React.FC = () => {
  return (
    <AuthStack.Navigator
      screenOptions={{
        headerShown: false,
        contentStyle: { backgroundColor: '#0a0d1a' },
      }}
    >
      <AuthStack.Screen name="PinEntry" component={LockScreen} />
      <AuthStack.Screen name="BiometricSetup" component={BiometricSetupScreen} />
      <AuthStack.Screen name="PinSetup" component={LockScreen} />
      <AuthStack.Screen name="Onboarding" component={HomeScreen} />
    </AuthStack.Navigator>
  );
};

export const MainTabNavigator: React.FC = () => {
  return (
    <MainTabs.Navigator
      screenOptions={{
        headerShown: true,
        headerStyle: { backgroundColor: '#0f172a' },
        headerTintColor: '#f8fafc',
        tabBarStyle: {
          backgroundColor: '#0f172a',
          borderTopColor: '#1e293b',
        },
        tabBarActiveTintColor: '#38bdf8',
        tabBarInactiveTintColor: '#64748b',
      }}
    >
      <MainTabs.Screen
        name="Fleet"
        component={HomeScreen}
        options={{ title: 'Fleet' }}
      />
      <MainTabs.Screen
        name="Market"
        component={WalletScreen}
        options={{ title: 'Wallet' }}
      />
      <MainTabs.Screen
        name="Explore"
        component={HomeScreen}
        options={{ title: 'Explore' }}
      />
      <MainTabs.Screen
        name="Guild"
        component={HomeScreen}
        options={{ title: 'Guild' }}
      />
      <MainTabs.Screen
        name="Settings"
        component={SettingsScreen}
        options={{ title: 'Settings' }}
      />
    </MainTabs.Navigator>
  );
};

export const AppNavigator: React.FC = () => {
  const { isLocked, setIsLocked } = useAppStore();

  useEffect(() => {
    LockManager.init(() => {
      setIsLocked(true);
    });

    return () => {
      LockManager.destroy();
    };
  }, [setIsLocked]);

  return (
    <NavigationContainer>
      <RootStack.Navigator screenOptions={{ headerShown: false }}>
        {isLocked ? (
          <RootStack.Screen name="AuthStack" component={AuthNavigator} />
        ) : (
          <RootStack.Screen name="MainTabs" component={MainTabNavigator} />
        )}
      </RootStack.Navigator>
    </NavigationContainer>
  );
};

export default AppNavigator;
